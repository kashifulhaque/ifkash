// The workout page now folds in the tracker, which relies on browser-only APIs
// (Google Identity Services, localStorage) and a signed-in token, so disable SSR.
export const ssr = false;
