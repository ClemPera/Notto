import { useEffect, useState } from "react";
import { useGeneral } from "../store/general"
import { invoke } from "@tauri-apps/api/core";

type User = {
  username: string,
  id: number
}

export default function Login() {
  const { userId, setUserId } = useGeneral();
  
  const [users, setUsers] = useState<User[]|null>(null)

  useEffect(() => {
    invoke("get_users").then((users) => setUsers(users as User[]))
      .catch((e) => console.error(e));
  }, [])

  return (
    <div className="flex">
      <h3 className="text-xl">Select the current user</h3>
      <div>
        {users && users.map((user) => (
          <div key={user.id} onClick={() => setUserId(user.id)}>{user.username}: {user.id}</div>
        ))}
      </div>
    </div>
  )
}