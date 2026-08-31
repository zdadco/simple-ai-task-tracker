import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppNav from "./components/AppNav";
import QuickCapture from "./windows/QuickCapture";
import TaskList from "./windows/TaskList";
import Settings from "./windows/Settings";
import Digests from "./windows/Digests";
import {
  APP_NAVIGATE_EVENT,
  isAppRoute,
  type AppNavigatePayload,
  type AppRoute,
} from "./lib/nav";

function MainApp() {
  const [route, setRoute] = useState<AppRoute>("tasks");

  useEffect(() => {
    const unlisten = listen<AppNavigatePayload>(APP_NAVIGATE_EVENT, (event) => {
      if (isAppRoute(event.payload?.route)) {
        setRoute(event.payload.route);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex h-screen flex-col bg-gray-50">
      <AppNav route={route} onNavigate={setRoute} />
      <div className="min-h-0 flex-1 overflow-hidden">
        {route === "tasks" && <TaskList />}
        {route === "digests" && <Digests />}
        {route === "settings" && <Settings />}
      </div>
    </div>
  );
}

function App() {
  const [windowLabel, setWindowLabel] = useState<string>("main");

  useEffect(() => {
    setWindowLabel(getCurrentWindow().label);
  }, []);

  if (windowLabel === "quick-capture") {
    return <QuickCapture />;
  }

  return <MainApp />;
}

export default App;
