# DOA BlackJack compatibility fork

This branch extends touchHLE v0.3.0 (base commit
`8eb3418b3343bc1e3d1a330b55cb1d708a015721`) with experimental support for
DOABlackjack 1.1, bundle identifier `jp.co.tecmo.DOABlackjack`, iPhone OS 2.2.1.
The changes were developed with AI assistance for this fork.

## Changes

- Register `UIBarItem`, `UIBarButtonItem` and `UIToolbar` in UIKit's host class
  exports. Toolbar buttons use the existing UIButton implementation, including
  target/action callbacks with the bar item as sender.
- Support item lists, title/system items, fixed and flexible spacing, basic
  layout, sizing, enabled state and a simple toolbar background.
- Implement CGContext global alpha, including graphics-state save/restore and
  drawing with straight or premultiplied alpha. Clearing ignores global alpha.
- Export user-to-device coordinate transformation and the bounding box of the
  currently supported full-bitmap clip region.
- Implement premultiplied Source In and Source Atop compositing.

The source changes are integrated in this branch. A standalone patch against
the base commit is also included at `patches/doa-blackjack.patch`.

## Build and run on Windows

Follow [building.md](building.md) for prerequisites: Rust, Visual Studio C++
tools, CMake and Boost 1.81 headers in `vendor/boost`.

```powershell
git submodule update --init --recursive
# Compatibility with newer CMake versions and older bundled dependencies:
$env:CMAKE_POLICY_VERSION_MINIMUM = '3.5'
cargo build --locked --release --bin touchHLE
cargo test --locked --release --lib
cargo clippy --locked --release --bin touchHLE -- --deny warnings
.\target\release\touchHLE.exe 'C:\path\to\DOABlackjack.ipa'
```

Run from the repository root so touchHLE can find the included fonts and guest
libraries. The game IPA must be supplied separately. No game files, extracted
assets, saves, compiler installations or build outputs are included in this fork.

## Validation (2026-10-29)

- Windows x64 release build successful; 33 Rust library tests passed.
- Clippy with warnings denied passed.
- Tested title screen, starting a game, betting, dealing, help pages, returning
  from help, and standing by swiping the table.
- Completed a round: 100 coins, bet 1, player stands on 12, dealer busts,
  WIN displayed, balance becomes 101 and bet resets to 0.
- Returning to the title and opening/leaving the album worked.

## Limitations

This is partial UIKit support: no original toolbar gradients or system icons,
animated item transitions, actual translucency, responder-chain actions for
nil targets, or NIB decoding. System buttons may use text labels. Changing an
individual item's width does not automatically request toolbar layout.
General clipping beyond the previously supported full-bitmap extent remains
unsupported. Toolbar initialization and item construction run in the game;
photo/web toolbar button callbacks have not been separately verified.

Audio, persistent saves, photo export, web content, insurance, splits and all
costumes have not been comprehensively tested. A completed round is not a
claim of complete game compatibility.

## API references

- [UIToolbar](https://developer.apple.com/documentation/uikit/uitoolbar?language=objc)
- [UIBarButtonItem](https://developer.apple.com/documentation/uikit/uibarbuttonitem)
- [Global alpha](https://developer.apple.com/documentation/coregraphics/cgcontext/setalpha(_:))
- [Source In](https://developer.apple.com/documentation/coregraphics/cgblendmode/sourcein)
- [Source Atop](https://developer.apple.com/documentation/coregraphics/cgblendmode/sourceatop)
