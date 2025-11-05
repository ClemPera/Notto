import { useEffect, useState } from "react";
import { useGeneral } from "../store/general"
import { invoke } from "@tauri-apps/api/core";

type User = {
  id: number
  username: string,
}

export default function Login() {
  const { userId, setUserId } = useGeneral();
  
  const [users, setUsers] = useState<User[]|null>(null);

  useEffect(() => {
    invoke("get_users").then((users) => setUsers(users as User[]))
      .catch((e) => console.error(e));
  }, [])

  async function selectUser(id: number) {
    invoke("set_user", { id: id }).catch((e) => console.error(e));

    setUserId(id);
  }

  async function create_account() {
    await invoke("create_account", { username: "bonjour", password: "aurevoir" }).then(v => console.info(v)).catch((e) => console.error(e));
  }

  return (
    <div className="flex flex-col gap-1">
      <button className="h-10 w-min p-2 bg-red-600 cursor-pointer" onClick={create_account}>create_account</button>

      <h3 className="text-xl">Select the current user</h3>
      <div className="flex flex-row gap-1">
        {users && users.map((user) => (
          <div key={user.id} 
            onClick={() => selectUser(user.id)}
            className="bg-amber-600 p-2 cursor-pointer">
              {user.username}: {user.id}
          </div>
        ))}
      </div>
    </div>
  )
}