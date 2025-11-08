// import { useEffect, useState } from "react";
// import { useGeneral } from "../../store/general"
// import { invoke } from "@tauri-apps/api/core";
// import CreateAccount from "./CreateAccount";

// type User = {
//   id: number
//   username: string,
// }

// export default function LoginHome() {
//   const { userId, setUserId } = useGeneral();
  
//   const [users, setUsers] = useState<User[]|null>(null);

//   const [createAccount, setCreateAccount] = useState<boolean>(false);

//   useEffect(() => {
//     invoke("get_users").then((users) => setUsers(users as User[]))
//       .catch((e) => console.error(e));
//   }, [])

//   async function selectUser(id: number) {
//     invoke("set_user", { id: id }).catch((e) => console.error(e));

//     setUserId(id);
//   }

  // return (
  //   <div>
  //     <button className="h-10 w-min p-2 bg-red-600 cursor-pointer" onClick={() => setCreateAccount(true)}>create_account</button>

  //     {/* {
  //       createAccount ? <CreateAccount/>
  //       : <Login/>
  //     } */}

  //   </div>
  // )
// }