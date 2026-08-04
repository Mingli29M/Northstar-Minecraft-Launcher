import { Navigate, Route, Routes } from "react-router-dom";
import { HomePage } from "./pages/HomePage";
import { AboutPage } from "./pages/AboutPage";
import { LicensePage } from "./pages/LicensePage";

export function App() {
  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      <Route path="/about" element={<AboutPage />} />
      <Route path="/license" element={<LicensePage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
