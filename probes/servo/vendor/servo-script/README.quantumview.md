# Local patch to servo-script 0.7.0

Source: the published `servo-script 0.7.0` crate. Its VCS metadata identifies
Servo commit `aac43a3f31a259f04a574f5ec4e959c943ad7cc7`, `components/script`.
The upstream source retains its MPL-2.0 headers and normalized Cargo manifest.
Registry metadata, the original manifest and the crate's nested lockfile are omitted.

The only Rust change is in `dom/html/form_controls/text_input.rs`:

- Track whether the current selection is IME preedit text.
- When an empty composition update replaces that text, remove the previous
  preedit selection and dispatch the updated input value.
- Ignore empty updates when there is no preedit selection, preserving an
  ordinary selection. Clearing the selection also clears the preedit marker.

This fixes the reported case where deleting an uncommitted character with
Backspace leaves the character visible until Backspace is pressed again.
It applies to native single-line and multiline text controls; the probe's
HTML/JavaScript does not contain an IME workaround.

The complete crate is included because Cargo patches resolve a whole package.
The version and dependency versions remain fixed. Only this package's registry
source/checksum are removed from the application's lockfile.
