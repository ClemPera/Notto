import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { useGeneral } from "./store/general";
import Home from "./components/Home";

function App() {
  const { userId, setUserId } = useGeneral();
  const user_id_to_change = 1;
  
  // async function test() {
  //   await invoke("test", {  }).then(v => console.info(v)).catch((e) => console.error(e));
  // }
  
  useEffect(() => {
    invoke("init").catch((e) => console.error(e));
    invoke("set_user", { id: user_id_to_change }).catch((e) => console.error(e));
    
    setUserId(user_id_to_change);
  }, [userId])

  return (
      <div>
        {userId && <Home/> }
  
        {/* <button className="h-10 w-20 bg-slate-600" onClick={test}>test</button> */}
      </div>
  );
}

export default App;
