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
  async function create_local_user() {
    await invoke("create_user", { username: "test_account" }).then(v => console.info(v)).catch((e) => console.error(e));
  }
  
  useEffect(() => {
    invoke("init").catch((e) => console.error(e));
    invoke("set_user", { id: user_id_to_change })
      .then(() => setUserId(user_id_to_change))
      .catch((e) => console.error(e));
    
  }, [userId])

  return (
      <div>
        {userId ? <Home/> : <button className="h-10 w-min p-2 bg-green-600 cursor-pointer" onClick={create_local_user}>create_local_user</button> }

        {/* TODO: handle login and create_user */}
        
        {/* <button className="h-10 w-20 bg-slate-600" onClick={test}>test</button> */}
      </div>
  );
}

export default App;
