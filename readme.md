# Goymusic/Tider's DAW

A small desktop tool with two parts:

- **Improviser**: generates random notes from a scale at a chosen tempo and sends them as MIDI through a virtual MIDI port.
- **Text to speech**: turns typed text into speech (eSpeak NG or Fish Audio) and plays it into a **virtual microphone**, so apps like OBS, Discord or a browser can use it as an input device.

Built with Rust and egui (eframe, glow renderer). Primary target: **Linux with PipeWire** (tested on KDE Plasma / Wayland).

> WIP atm on other platforms and features other features

## Requirements

| What | Why |
|---|---|
| Rust (stable, edition 2024) | building the app |
| `pactl` and `pacat` (package `pulseaudio-utils` or `libpulse`) | creating and feeding the virtual microphone |
| PipeWire with `pipewire-pulse` (default on KDE) | the virtual mic is a PipeWire/Pulse null sink + remap source |
| ALSA sequencer (`alsa-lib`, `alsa-utils`) | the virtual MIDI port |
| `espeak-ng` | the eSpeak TTS backend |
| `xdg-desktop-portal` + `xdg-desktop-portal-kde` | global hotkey on Wayland |
| A Fish Audio API key *(optional)* | the Fish Audio TTS backend |

Install the system packages with your distro's package manager, for example on Arch:

```bash
sudo pacman -S rust pulseaudio-libs alsa-utils espeak-ng xdg-desktop-portal-kde
```

(On Debian/Ubuntu the equivalents are `libpulse0`/`pulseaudio-utils`, `alsa-utils`, `espeak-ng` and `xdg-desktop-portal-kde`.)

## Installation

From the repository root:

```bash
chmod +x install.sh
FISH_API_KEY=your_key_here ./install.sh
```

The Fish key is optional. You can also run `./install.sh` with no key, pass it as `./install.sh --fish-key your_key_here`, or type it into the TTS screen later. Passing it through the environment variable is preferred, because command-line arguments end up in shell history and `ps`.

Make sure `~/.cargo/bin` is in your `PATH` if you want to run `goymusic` from a terminal.

### Uninstall

```bash
./install.sh --uninstall
```

This removes the binary, the launcher and the desktop entry. Your key file is left in place; delete `~/.config/goymusic/` yourself if you want it gone.

### Running without installing

```bash
cargo run --release
```

Global hotkeys will not work this way (the portal needs the desktop entry), but everything else does.

## Usage

On launch you get the **Launcher** window with one button per tool. Each tool opens in its own window. Press the button again to focus or close it.

### Improviser

1. Click **Open Improviser**.
2. Choose a **Root note**, **Scale**, **Note duration**, **Octave** and **Tempo**.
3. Press **Play** to start, **Pause** to stop (held notes are released on pause).

The app creates a virtual MIDI output (client `goymusic`, port `goymusic port`). Nothing makes sound until something is listening to it:

Or connect it graphically with `qpwgraph`, or select it as a MIDI input in your DAW. For a quick software synth: `fluidsynth -a pipewire /path/to/soundfont.sf2`, then connect the port to it. Use `aseqdump -p <client>:0` to check that notes are being sent.

### Text to speech

1. Click **Open TTS** in the launcher.
2. Pick a **backend** and adjust its parameters:
   - **eSpeak NG**: voice, speed, pitch. Works offline and instantly.
   - **Fish Audio**: API key, model, voice preset or a custom voice model ID. Needs internet and an account.
3. Type your text and press **Speak** (or **Enter**). **Ctrl+Enter** or **Shift+Enter** inserts a new line.
4. **Stop** cuts off the clip that is currently playing.

The speech is played into the virtual microphone `goymusic_mic`. You will not hear it through your speakers; to monitor it, enable monitoring in OBS or route `goymusic_mic_sink.monitor` to your output with `qpwgraph`.

#### Using it as a microphone

Select **goymusic_mic** as the input device in the target app. In OBS: add an **Audio Input Capture** source and choose `goymusic_mic`. The app must already be running so the device exists when you open the dropdown.

If KDE's volume widget does not show it, enable "Show virtual devices" in the widget's settings, or check with:

```bash
pactl list short sources
```

### Quick TTS prompt (global hotkey)

Press **Alt+T** from any application to open a small text prompt. **Enter** speaks the text and closes the prompt, **Esc** cancels.

On Wayland the shortcut goes through the XDG GlobalShortcuts portal. The first time, KDE asks you to confirm it. After that, the binding lives in **System Settings → Shortcuts → Goymusic**, and whatever you set there overrides Alt+T. If the app is not listed there, the registration did not go through (see Troubleshooting).

## Troubleshooting

**No `goymusic_mic` device.** Run the app from a terminal and look for `virtual mic unavailable: ...`. Check that `pactl` and `pacat` are installed and that `pipewire-pulse` is running. After a crash, stale modules can block the next start; remove them with:

```bash
pactl list short modules | grep -E 'goymusic'
pactl unload-module <id>     # unload the remap-source first, then the null-sink
```

(Avoid `pactl unload-module module-null-sink` by name; it unloads every null sink on the system.)

**Alt+T does nothing.**
- Make sure you installed with `install.sh` and started the app from the launcher entry, not only via `cargo run`.
- The terminal prints `global hotkeys: xdg-desktop-portal` when the portal backend is active, and `none` when it failed.
- `An app id is required` / `App info not found` means the desktop entry is missing or not yet picked up. Run `kbuildsycoca6`, or log out and back in.
- Check System Settings → Shortcuts for the "Goymusic" entry.

**The prompt window has no keyboard focus.** Wayland compositors decide focus. Click the text box once. A KDE window rule for the "Quick TTS" window can force focus, position and "keep above".

**Fish Audio fails.** The error appears in the terminal as `TTS failed: ...`. Common causes are a missing or wrong API key, a wrong voice model ID (`Reference not found`), or a model your plan does not include.

**No sound from the improviser.** MIDI carries notes only. Connect the `goymusic port` output to a synth (see above).

## Other platforms

WIP

## Project layout

```
goymusic/
├── src/main.rs            entry point
└── crates/
    ├── improviser/        note generation (pure logic)
    ├── mididevice/        MIDI output backends
    ├── micdevice/         virtual microphone backends
    ├── tts/               speech synthesis backends (eSpeak NG, Fish Audio)
    ├── hotkeys/           global hotkeys (XDG portal on Wayland, global-hotkey elsewhere)
    ├── context/           owns the devices and runs the update loop
    └── frontend/          egui windows and screens
```