import { redirect } from "@sveltejs/kit";

export function load() {
  throw redirect(301, "https://gist.github.com/kashifulhaque/99fe8614efd5aac2694845132d1b3c07");
}
