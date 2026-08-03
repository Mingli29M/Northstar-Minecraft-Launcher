import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { Providers } from "./providers";
import App from "./App";
import { ConsoleWindowApp } from "./components/ConsolePanel";
import { isConsoleWindow } from "./lib/downloadStatus";
import "./index.css";

const root = createRoot(document.getElementById("root")!);

if (isConsoleWindow()) {
  root.render(
    <Providers>
      <ConsoleWindowApp />
    </Providers>,
  );
} else {
  root.render(
    <BrowserRouter>
      <Providers>
        <App />
      </Providers>
    </BrowserRouter>,
  );
}
