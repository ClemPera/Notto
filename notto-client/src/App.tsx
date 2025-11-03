import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Login from "./components/Login";
import { useGeneral } from "./store/general";

function App() {
  const { userId, setUserId } = useGeneral();
  
  // async function create_note() {
  //   await invoke("create_note", { title: "titre" }).catch((e) => console.error(e));
  // }

  // async function get_note() {
  //   await invoke("get_note", { id: 1 }).then(v => console.info(v)).catch((e) => console.error(e));
  // }

  // async function create_account() {
  //   await invoke("create_account", { username: "bonjour", password: "aurevoir" }).then(v => console.info(v)).catch((e) => console.error(e));
  // }

  // async function test() {
  //   await invoke("test", {  }).then(v => console.info(v)).catch((e) => console.error(e));
  // }
  
  // useEffect(() => {
  //   console.log(userId);
  //   // invoke("init").catch((e) => console.error(e));
  // }, [userId])

  return (
      <div>
        {userId ? "loggedIn(todo)" : <Login/> }
        {/* <button className="h-10 w-20 bg-amber-600" onClick={create_note}>create</button>
  
        <button className="h-10 w-20 bg-blue-600" onClick={get_note}>get</button>
  
        <button className="h-10 w-20 bg-red-600" onClick={create_account}>create_account</button>
  
        <button className="h-10 w-20 bg-slate-600" onClick={test}>test</button> */}
      </div>
  );
}

export default App;
