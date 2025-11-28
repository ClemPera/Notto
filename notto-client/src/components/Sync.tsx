import { useState } from "react";
import { useGeneral } from "../store/general";
import { invoke } from "@tauri-apps/api/core";
import { info } from "@tauri-apps/plugin-log";

enum SyncStatus {
  Ok,
  Syncing,
  Error,
}

export default function Sync() {
  const { userId, setUserId } = useGeneral();
  const [logged, setLogged] = useState<boolean>(false);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>(SyncStatus.Syncing);

  async function create_account() {
    await invoke("sync_create_account", { username: "test_account", password: "password" })
      .catch((e) => console.error(e));
  }

  async function login() {
    await invoke("sync_login", { username: "test_account", password: "password" })
      .then((loggedStatus) => {
        setLogged(() => loggedStatus as boolean);
        console.log("user has been logged: ", loggedStatus as boolean);
        console.log("logged: ", logged)
      })
      .catch((e) => console.error(e));
  }
  
  async function sync() {
    //TODO
  }

  return (
    <div>
      <h3 className="text-xl">Server actions</h3>
      <button className="h-10 w-min p-2 bg-yellow-600 cursor-pointer" onClick={create_account}>create_account</button>
      <button className="h-10 w-min p-2 bg-blue-600 cursor-pointer" onClick={login}>login</button>

      {logged ?? <div>
        <button className="h-10 w-min p-2 bg-green-600 cursor-pointer" onClick={sync}>sync_notes</button>
        <p>Sync status: {syncStatus}</p>
      </div>}

    </div>
  )
}
