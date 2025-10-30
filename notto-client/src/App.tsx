import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function create_note() {
    await invoke("create_note", { title: "titre" }).catch((e) => console.error(e));
  }

  async function get_note() {
    await invoke("get_note", { id: 1 }).then(v => console.info(v)).catch((e) => console.error(e));
  }

  return (
    <main className="container">
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <button className="h-10 w-20 bg-amber-600" onClick={create_note}>create</button>

      <button className="h-10 w-20 bg-blue-600" onClick={get_note}>get</button>
    </main>
  );
}

export default App;
