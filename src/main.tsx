import React from "react";
import { createRoot } from "react-dom/client";

// Tipografía autoalojada (R-03): sin CDN, sin FOUT en el widget.
import "@fontsource-variable/fraunces";
import "@fontsource/space-grotesk/400.css";
import "@fontsource/space-grotesk/500.css";
import "@fontsource/space-grotesk/700.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";

import "./styles/index.css";
import App from "./App";

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
