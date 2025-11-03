import { create } from "zustand"

type Store = {
  userId: number | null

  setUserId: (newUserId: number) => void
}

export const useGeneral = create<Store>(
  (set) => ({
    userId: null,

    setUserId: (newUserId) => {
      set(() => ({ userId: newUserId }))
    }
  })
)