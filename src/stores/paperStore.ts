import { writable } from "svelte/store";

export const paperImageUrl = writable<string | null>(null);
