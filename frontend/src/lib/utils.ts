import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// Generic helper that unwraps an error object coming from axios/fetch/etc.
// Intended for showing meaningful messages in toast notifications.
export function getErrorMessage(err: unknown): string {
  // Axios error: err is object with isAxiosError
  if (
    typeof err === "object" &&
    err !== null &&
    // @ts-expect-error – axios flag exists at runtime
    (err.isAxiosError || (err as any).response)
  ) {
    const axiosErr = err as any;
    // Prefer explicit backend-provided message fields
    const msg = axiosErr.response?.data?.message || axiosErr.response?.data?.error;
    if (msg) return msg as string;
    if (axiosErr.message) return axiosErr.message as string;
  }

  // Standard Error
  if (err instanceof Error) {
    return err.message;
  }

  // Fallback to string representation
  return String(err);
}
