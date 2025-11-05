import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Login from "./components/Login";
import { useGeneral } from "./store/general";
import Home from "./components/Home";

function App() {
  const { userId, setUserId } = useGeneral();
  
  // async function test() {
  //   await invoke("test", {  }).then(v => console.info(v)).catch((e) => console.error(e));
  // }
  
  useEffect(() => {
    console.log(userId);
    invoke("init").catch((e) => console.error(e));
  }, [userId])

  return (
      <div>
        {userId ? ( <Home/> ) : ( <Login/> ) }
  
        {/* <button className="h-10 w-20 bg-slate-600" onClick={test}>test</button> */}
      </div>
  );
}

export default App;
