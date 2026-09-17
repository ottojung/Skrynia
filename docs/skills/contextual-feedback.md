# Skrynia contextual feedback skill

## Scope

Use this skill when a Skrynia-hosted web app needs durable, spatially anchored textual feedback that can later be retrieved and understood without the original browser session.

The feedback UI is app-side code. Skrynia supplies only durable object storage through its normal store API.

## User interaction

Provide one round feedback button that is fixed to the viewport and remains visible while the user moves around the page.

When the user activates it:

1. open a text comment composer;
2. create a visible pointer at the current viewport center;
3. represent that pointer in absolute document coordinates (`documentX`, `documentY`);
4. let the user drag the pointer before saving;
5. show the feedback action row described below;
6. on save, persist the comment, pointer location, and recoverable HTML context together.

The feedback UI itself must be excluded from captured page context so feedback does not recursively snapshot its own controls or pointer marker.

Text input is ordinary browser text input. Phone dictation may be used by the user, but the app does not record or transcribe audio.

### Feedback action-row layout

The opened feedback panel has one dedicated action row immediately below the comment/help area. That row contains exactly these three peer buttons, in this left-to-right order:

```text
Copy JSON    Cancel    Save
```

`Copy JSON` must be visibly next to `Cancel` and `Save`. Do not place it in a separate menu, header, overflow control, secondary panel, or status area.

Any transient status text such as `Copying…`, `Copied JSON`, `Saving…`, or an error message must be rendered on its own line outside the action row. Status text must never consume horizontal space that could hide, wrap away, or displace `Copy JSON`.

On narrow mobile viewports, all three action buttons must remain simultaneously visible without horizontal scrolling. Prefer a three-column grid or an equivalent layout that reserves one visible slot for each button. Do not rely on a wrapping flex row where one button may move out of the visible area.

### Copy JSON behavior

The **Copy JSON** button is available whenever the feedback panel is open and is independent of whether the current comment field contains text.

On activation:

1. perform a fresh `GET` of the standard `feedback-v1` object with browser caching disabled;
2. require a successful response;
3. copy the exact returned JSON text to the system clipboard;
4. report success or failure in the separate feedback-status line without changing the stored object.

Prefer `navigator.clipboard.writeText(...)` in secure contexts. A compatibility fallback may be used when the Clipboard API is unavailable, but it must still copy the same fetched text. Do not synthesize a replacement object from stale in-memory state when the persisted object is readable.

## Standard storage key

All feedback for one Skrynia app namespace is stored as one JSON object at the standard key:

```text
feedback-v1
```

For namespace `example`, the public storage URL is therefore:

```text
{SKRYNIA_URL}/store/example/feedback-v1
```

Create the object as `public-write` when the feedback UI is a static browser app with no write credential. Do not put credentials into shipped JavaScript.

The canonical object shape is:

```json
{
  "version": 1,
  "updatedAt": "2026-09-17T00:00:00.000Z",
  "htmlFragments": {
    "sha256:...": "<div>...</div>",
    "sha256:...": "<body>...</body>"
  },
  "feedback": [
    {
      "id": "...",
      "text": "...",
      "createdAt": "...",
      "pagePath": "/a/example/",
      "pointer": {
        "documentX": 812.5,
        "documentY": 2940.0,
        "documentWidth": 1180,
        "documentHeight": 6412,
        "viewportWidth": 412,
        "viewportHeight": 915,
        "scrollX": 0,
        "scrollY": 2482
      },
      "context": {
        "chain": [
          "sha256:leaf-div",
          "sha256:parent-div",
          "sha256:body"
        ]
      }
    }
  ]
}
```

`htmlFragments` is a content-addressed dictionary. Hash the exact serialized fragment bytes with SHA-256 and use the hash as the key. If an identical fragment is encountered again, reference its existing hash instead of storing the HTML again.

## Capturing pointer context

At save time, resolve the DOM content underneath the pointer, not the feedback overlay itself.

1. Convert the absolute document point back to viewport coordinates using the current `scrollX` and `scrollY`.
2. Temporarily make the feedback pointer/overlay non-hit-testable, or use another equivalent technique, then call `document.elementFromPoint`.
3. From the hit element, find the narrowest enclosing `div`.
4. Walk upward through enclosing `div` ancestors.
5. Append `document.body` as the final context element, even when there is no enclosing `div`.

The resulting `context.chain` is ordered from narrowest context to broadest context and always ends at the captured body.

Each element in that chain must be serialized from a cloned DOM with feedback UI nodes removed. The serialized body must be sufficient to recover the full current HTML body at save time. Runtime DOM changes made by the application should therefore be reflected in the snapshot.

The same page fragment can occur in many feedback contexts. Store each exact serialized fragment once in `htmlFragments`; feedback entries contain only hash references.

## Pointer coordinates

`documentX` and `documentY` are CSS-pixel coordinates relative to the document origin at save time:

```text
documentX = viewportX + window.scrollX
documentY = viewportY + window.scrollY
```

Also save document dimensions, viewport dimensions, and scroll position. Those fields are not a replacement for the absolute coordinates; they make later interpretation and debugging unambiguous.

The pointer must be draggable with Pointer Events so the same behavior works for mouse, pen, and touch.

## Persistence behavior

Before writing, read the current `feedback-v1` object and merge the new entry into it. This reduces accidental overwrites when a page has been open for a long time. Skrynia public-write storage does not provide compare-and-swap, so concurrent writers can still race; do not pretend otherwise.

On a missing object:

- initialize `{ "version": 1, "updatedAt": null, "htmlFragments": {}, "feedback": [] }`;
- create it with `POST` and `X-Skrynia-Mode: public-write`.

On an existing object, replace it with `PUT`.

Never overwrite earlier feedback merely because a new pointer happens to occupy the same location.

## Retrieval contract

A later reviewer should need only the one `feedback-v1` object to understand all current feedback.

For each feedback entry:

- read `text`;
- inspect `pointer.documentX` / `pointer.documentY`;
- resolve every hash in `context.chain` through `htmlFragments`;
- use the first fragment as the narrowest enclosing context and the last fragment as the full captured body.

This representation is intentionally independent of fragile CSS selectors or element IDs. IDs and classes remain useful because they are preserved inside the captured HTML, but retrieval correctness does not depend on them.

The UI's Copy JSON action is the user-facing export path for this same object. It does not define a second representation.

## Deployment verification

A deployment that uses this skill should verify at least:

- the production bundle contains the feedback code;
- the opened feedback panel visibly presents `Copy JSON`, `Cancel`, and `Save` together in that order;
- the `feedback-v1` object exists and parses as JSON with `version === 1`;
- `htmlFragments` is an object and `feedback` is an array;
- the app page and Skrynia health endpoint are reachable.

Do not make smoke tests depend on JSON whitespace formatting.
