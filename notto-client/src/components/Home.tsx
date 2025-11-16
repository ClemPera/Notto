import { useEffect, useState } from "react";
import { useGeneral } from "../store/general";
import { invoke } from "@tauri-apps/api/core";
import Sync from "./Sync";

type Note = {
  id: number
  title: string,
  created_at: Date,
}

type NoteContent = {
  id: number
  title: string,
  content: string,
  created_at: Date,
}

export default function Home() {
  const { userId, setUserId } = useGeneral();
  const [notes, setNotes] = useState<Note[]|null>(null);
  const [currentNote, setCurrentNote] = useState<NoteContent|null>();
  
  useEffect(() => {
    get_notes_metadata();
  }, [])

  function get_notes_metadata() {
    invoke("get_all_notes_metadata", {id_user: userId}).then((notes) => setNotes(notes as Note[]))
      .catch((e) => console.error(e));
  }

  async function create_note() {
    await invoke("create_note", { title: "titre" }).catch((e) => console.error(e));
    get_notes_metadata();
  }

  async function get_note(id: number) {
    await invoke("get_note", { id: id }).then((note) => setCurrentNote(note as NoteContent)).catch((e) => console.error(e));
  }

  async function edit_note(content: string) {
    const note: NoteContent = {
      id: currentNote?.id!,
      title: currentNote?.title!,
      created_at: currentNote?.created_at!,
      content: content,
    }
    
    setCurrentNote(note);

    await invoke("edit_note", { note }).catch((e) => console.error(e));
  }

  async function edit_note_title(title: string) {
    const note: NoteContent = {
      id: currentNote?.id!,
      title: title!,
      created_at: currentNote?.created_at!,
      content: currentNote?.content!,
    }
    
    setCurrentNote(note);
    
    await invoke("edit_note", { note }).catch((e) => console.error(e));
    
    get_notes_metadata();
  }

  return (
    <div className="flex flex-row">
      <div className="flex flex-col">
        <button className="h-10 w-min p-2 bg-green-600 cursor-pointer" onClick={create_note}>create_note</button>

        <h3 className="text-xl">Here's your notes</h3>
        <div className="flex flex-col gap-1">
          {notes && notes.map((note) => (
            <div key={note.id}
              className="bg-amber-600 p-2 cursor-pointer"
              onClick={() => get_note(note.id)}>
                {note.title}, {note.created_at.toString()}
            </div>
          ))}
        </div>
        <br/>
        <Sync/>
      </div>
        {currentNote ? (
          <div className="flex flex-col grow">
            <input type="text" className="text-xl" onChange={(e) => edit_note_title(e.target.value)} value={currentNote.title} ></input>
            <textarea className="h-full bg-gray-500" onChange={(e) => edit_note(e.target.value)} value={currentNote.content}></textarea>
          </div>
        ) : ""}
    </div>
  )
}