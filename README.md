# gigabyte-kbd-backlight

Utility for controlling keyboard backlight on Gigabyte G6X9MG laptops via direct EC (Embedded Controller) register writes through `/dev/port`.

Architecture: privileged daemon + unprivileged client communicating over a Unix socket (`/run/kbdlight.sock`).

Tested on Fedora 43 (kernel 6.18.x).

## Build

```
cargo build --release
```

## Setup

Create the `kbdlight` group and add your user to it:

```
sudo groupadd kbdlight
sudo usermod -aG kbdlight $USER
```

Copy the binary:

```
sudo cp target/release/gigabyte-kbd-backlight /usr/local/bin/
```

## Usage

Start the daemon (requires root):

```
sudo gigabyte-kbd-backlight daemon
```

Client commands (requires membership in `kbdlight` group):

```
gigabyte-kbd-backlight on                     # turn on (max brightness)
gigabyte-kbd-backlight off                    # turn off
gigabyte-kbd-backlight set-color FF0000       # set color (hex RRGGBB or #RRGGBB)
gigabyte-kbd-backlight set-brightness 5       # set brightness (0-9)
gigabyte-kbd-backlight adjust-brightness -2   # adjust brightness by delta
```

## Systemd Service

Create `/etc/systemd/system/gigabyte-kbd-backlight.service`:

```ini
[Unit]
Description=Gigabyte keyboard backlight daemon
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/local/bin/gigabyte-kbd-backlight daemon
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
```

Enable and start:

```
sudo systemctl enable --now gigabyte-kbd-backlight
```

## Technical Reference

Protocol reverse-engineered from DSDT disassembly (`/sys/firmware/acpi/tables/DSDT`) and empirical testing on my Gigabyte G6X9MG.

This section is for those who want to dig deeper or adapt the utility for other Gigabyte models.

### Hardware

| Parameter      | Value                                            |
|----------------|--------------------------------------------------|
| Laptop         | GIGABYTE G6X9MG                                 |
| DSDT           | GBT / GBTUACPI, revision 2                      |
| Keyboard       | PS/2 via i8042 (NOT USB)                         |
| Backlight type | RGB, single zone (whole keyboard)                |
| Control path   | Embedded Controller (EC) via I/O ports 0x62/0x66 |
| WMI GUID       | `ABBC0F6D-8EA1-11D1-00A0-C90629100000` (method "BB") |

There is no ITE USB HID device. The backlight is controlled entirely through the EC, which the Windows "GIGABYTE Control Center" accesses via WMI method calls (WMBB). Under Linux there is no driver for this WMI GUID, so we communicate with the EC directly through `/dev/port`.

### EC Communication Protocol

#### I/O Ports

| Port | Function                             |
|------|--------------------------------------|
| 0x62 | EC data port (read/write data bytes) |
| 0x66 | EC command/status port               |

#### Status Register (port 0x66, read)

| Bit | Name | Meaning                                        |
|-----|------|------------------------------------------------|
| 0   | OBF  | Output Buffer Full — data ready to read        |
| 1   | IBF  | Input Buffer Full — EC busy, wait before write |

#### Write to EC Register

To write value `val` at EC register offset `addr`:

```
1. Poll port 0x66 until bit 1 (IBF) = 0
2. Write 0x81 to port 0x66          (ACPI "Write Embedded Controller" command)
3. Poll port 0x66 until IBF = 0
4. Write addr to port 0x62           (register offset)
5. Poll port 0x66 until IBF = 0
6. Write val to port 0x62            (data byte)
```

#### Read from EC Register

To read EC register at offset `addr`:

```
1. Poll port 0x66 until IBF = 0
2. Write 0x80 to port 0x66          (ACPI "Read Embedded Controller" command)
3. Poll port 0x66 until IBF = 0
4. Write addr to port 0x62           (register offset)
5. Poll port 0x66 until OBF = 1
6. Read value from port 0x62
```

### Command Interface Registers (offset 0xF8-0xFD)

| Offset | Name | Description                                              |
|--------|------|----------------------------------------------------------|
| 0xF8   | FCMD | Command register — **write last**, triggers EC execution |
| 0xF9   | FDAT | First data byte (sub-command / channel selector)         |
| 0xFA   | FBUF | Second data byte                                         |
| 0xFB   | FBF1 | Third data byte                                          |
| 0xFC   | FBF2 | Fourth data byte                                         |
| 0xFD   | FBF3 | Fifth data byte                                          |

**Write order**: FDAT, FBUF, FBF1, FBF2, FBF3 first, then **FCMD last** (writing FCMD triggers command execution in the EC firmware).

### Working Commands

Only two commands have been confirmed to work on the G6X9MG:

| Feature    | EC cmd | FDAT | FBUF / FBF1 / FBF2 |
|------------|--------|------|---------------------|
| Brightness | 0xC4   | 0x02 | 0x00-0xFF           |
| Color      | 0xCA   | 0x03 | B, R, G             |

#### Set Color (EC cmd 0xCA, FDAT=0x03)

```
FDAT = 0x03
FBUF = Blue
FBF1 = Red
FBF2 = Green
FCMD = 0xCA
```

Note the BGR byte order. This was determined empirically — DSDT locals don't reflect the actual hardware mapping.

The DSDT defines 4 zones (FDAT sub-commands 0x03, 0x04, 0x05, 0x07), but only sub-command 0x03 has any visible effect on the G6X9MG. It sets the color of the entire keyboard uniformly. The G6X9MG has a single-zone backlight.

#### Set Brightness (EC cmd 0xC4, FDAT=0x02)

```
FDAT = 0x02
FBUF = brightness (0x00 = off, 0xFF = max)
FCMD = 0xC4
```

The DSDT formula `0xFF - (level * 0x19)` has an inverted scale (level 0 = brightest). This utility uses level 0 = off, level 9 = max instead.

### Non-working Commands (on G6X9MG)

These commands exist in the DSDT but had no visible effect or wrong behavior on this model:

| Command | FDAT       | DSDT purpose     | Result                       |
|---------|------------|------------------|------------------------------|
| 0xC4    | 0x03       | Effect/animation | No visible effect            |
| 0xC4    | 0x04       | Animation speed  | No visible effect            |
| 0xC4    | 0x07-0x0B  | Preset modes     | No visible effect            |
| 0xC4    | 0x0D/0x0E  | Turn on/off      | No visible effect            |
| 0xCA    | 0x00       | LED mode         | EC accepts, no visible change|
| 0xC1    | any        | Per-channel color | Controls fan, NOT keyboard   |

### WMI Interface

The WMI method "BB" (GUID `ABBC0F6D-8EA1-11D1-00A0-C90629100000`) dispatches to:

- **GCMD** — read/query operations
- **SCMD** — write/set operations

SCMD 0x67 is the main LED control sub-command. Its argument is a 32-bit bitfield:

```
Bits 31-28: Function selector (0x00-0x0F)
Bits 27-24: Sub-parameter (zone, pattern index)
Bits 23-16: Data byte
Bits 15-12: Brightness level (0-9)
Bits 11-0:  Additional color data
```

The two working functions are:
- **0x0D** — set brightness (EC cmd 0xC4, FDAT=0x02)
- **0x0F** — set per-zone RGB color (EC cmd 0xCA)

### Notes

Direct port I/O can race with the kernel's ACPI EC driver, which also uses ports 0x62/0x66. In practice no issues were observed, but the proper solution would be a kernel module calling `wmi_evaluate_method()` with the appropriate GUID.

Access to `/dev/port` requires root privileges or `CAP_SYS_RAWIO`.

## License

This project is licensed under the [GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.html).

## Author

- [Winterhearted](https://byte0.org) (byte0.org)
