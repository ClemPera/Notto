import { useState } from "react";
import { useGeneral } from "../store/general";
import { invoke } from "@tauri-apps/api/core";

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
    await invoke("sync_create_account", {id_user: userId}) //.then((notes) => setNotes(notes as Note[]))
      .catch((e) => console.error(e));
  }

  async function login() {
    //TODO
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
