/**
 * Turns a rejected command or a thrown value into a message a reader can read.
 *
 * Tauri rejects with a plain string, browsers with an Error; anything else falls
 * back to the caller's own wording so the status line is never empty.
 */
export function readableError(error: unknown, fallback: string) {
  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  return fallback;
}
