// The meal tracker relies on browser-only APIs (Google Identity Services,
// localStorage, canvas image downscaling, the File/camera API) and a signed-in
// token, so disable SSR for this route.
export const ssr = false;
