# Screenshots

The Windows images were captured from the running application's viewport using
iced's native `window::screenshot` API. They show the actual chooser UI and the
browser icons extracted from the installed Windows applications.

| Image | Setup |
| --- | --- |
| `windows-chooser.png` | Chrome and Edge installed, default browser order, `https://example.com` |
| `windows-browser-order.png` | Same URL and browsers, with `order = ["Microsoft Edge", "Google Chrome"]` |

Both captures use the existing application view. Only the capture trigger and
image export were added to a temporary binary; no capture functionality is
included in the distributed application.

To refresh these images, open the chooser with `https://example.com`, capture
the app window, and save the PNG here. Keep the URL, browser names, and keyboard
shortcuts legible. Check the rendered README after replacing an image.

Linux screenshots can be added here later. macOS screenshots require a macOS
implementation first; the application currently supports Linux and Windows.
