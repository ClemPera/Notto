import { useEffect, useState } from "react";
import { useGeneral } from "../store/general";
import { invoke } from "@tauri-apps/api/core";

type Note = {
  id: number
  title: string,
  created_at: Date,
}

export default function Home() {
  const { userId, setUserId } = useGeneral();
  
  const [notes, setNotes] = useState<Note[]|null>(null);
  
  useEffect(() => {
    invoke("get_all_notes_metadata", {id_user: userId}).then((notes) => setNotes(notes as Note[]))
      .catch((e) => console.error(e));
  }, [])

  return (
    <div className="flex flex-col">
      <h3 className="text-xl">Here's your notes</h3>
      <div className="flex flex-col gap-1">
        {notes && notes.map((note) => (
          <div key={note.id}
            className="bg-amber-600">
              {note.title}, {note.created_at.toString()}
          </div>
        ))}
      </div>
    </div>
  )
}