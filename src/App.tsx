import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import QuickCapture from "./windows/QuickCapture";
import TaskList from "./windows/TaskList";
import Settings from "./windows/Settings";

function App() {
  const [windowLabel, setWindowLabel] = useState<string>("main");

  useEffect(() => {
    setWindowLabel(getCurrentWindow().label);
  }, []);

  switch (windowLabel) {
    case "quick-capture":
      return <QuickCapture />;
    case "settings":
      return <Settings />;
    default:
      return <TaskList />;
  }
}

export default App;
