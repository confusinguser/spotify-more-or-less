"use client"

import { useState, useCallback, useEffect } from "react"
import { SongCard } from "./song-card"
import { motion } from "framer-motion"
import { fetchClient, TrackInfo } from "@/lib/api"
import { components } from "@/lib/schema"
import { Song } from "@/lib/types"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

type TrackInfo = components["schemas"]["TrackInfo"]

function trackInfoToSong(track: TrackInfo): Song {
  return {
    id: track.spotify_url || Math.random().toString(),
    title: track.title,
    artist: track.artist,
    streams: track.times_played,
    albumArt: track.album_image_url || "/placeholder-album.jpg",
    spotify_url: track.spotify_url,
    preview_url: track.preview_url || undefined,
  }
}

type AnimationType = "fling" | "flip" | "spiral" | "bounce" | "arc" | "somersault"

const dramaticAnimations: Record<
  AnimationType,
  {
    keyframes: { x: number[]; y: number[]; rotate?: number[]; rotateY?: number[]; rotateX?: number[]; scale: number[] }
    transition: object
  }
> = {
  fling: {
    keyframes: {
      x: [0, -100, -300, -400],
      y: [0, -150, -50, 0],
      rotate: [0, -15, 5, 0],
      scale: [1, 1.1, 1.05, 1],
    },
    transition: { duration: 0.8, ease: "easeOut", times: [0, 0.3, 0.7, 1] },
  },
  flip: {
    keyframes: {
      x: [0, -190, -400],
      y: [0, -50, 0],
      rotateY: [0, 180, 360],
      scale: [1, 1.2, 1],
    },
    transition: { duration: 0.9, ease: "easeInOut" },
  },
  spiral: {
    keyframes: {
      x: [0, -100, -250, -400],
      y: [0, -80, 80, 0],
      rotate: [0, 180, 540, 720],
      scale: [1, 0.8, 1.1, 1],
    },
    transition: { duration: 1, ease: "easeInOut", times: [0, 0.3, 0.7, 1] },
  },
  bounce: {
    keyframes: {
      x: [0, -120, -240, -320, -400],
      y: [0, -120, 0, -60, 0],
      scale: [1, 1.15, 0.95, 1.1, 1],
    },
    transition: { duration: 0.9, ease: "easeOut" },
  },
  arc: {
    keyframes: {
      x: [0, -190, -400],
      y: [0, -200, 0],
      rotate: [0, -10, 0],
      scale: [1, 1.15, 1],
    },
    transition: { duration: 0.8, ease: [0.33, 1, 0.68, 1] },
  },
  somersault: {
    keyframes: {
      x: [0, -100, -280, -400],
      y: [0, -100, -150, 0],
      rotateX: [0, 180, 360, 360],
      scale: [1, 1.1, 1.2, 1],
    },
    transition: { duration: 0.95, ease: "easeInOut" },
  },
}

const animationTypes: AnimationType[] = ["fling", "flip", "spiral", "bounce", "arc", "somersault"]

function getRandomAnimation(): AnimationType {
  return animationTypes[Math.floor(Math.random() * animationTypes.length)]
}

// Alliteration map: letter -> [adjective, noun]
const ALLITERATIONS: Record<string, [string, string]> = {
  A: ["Audacious", "Anthems"],
  B: ["Bangin'", "Beats"],
  C: ["Chaotic", "Choruses"],
  D: ["Dramatic", "Drops"],
  E: ["Easy", "Echos"],
  F: ["Frantic", "Frequencies"],
  G: ["Groovy", "Grooves"],
  H: ["Hypnotic", "Harmonies"],
  I: ["Insane", "Interludes"],
  J: ["Jazzy", "Jams"],
  K: ["Kinetic", "Keys"],
  L: ["Loud", "Lyrics"],
  M: ["Musical", "Machinations"],
  N: ["Nostalgic", "Notes"],
  O: ["Outrageous", "Overtones"],
  P: ["Pulsating", "Playlists"],
  Q: ["Quirky", "Quartets"],
  R: ["Raucous", "Rhythms"],
  S: ["Sonic", "Sonnets"],
  T: ["Thunderous", "Tracks"],
  U: ["Unhinged", "Undertones"],
  V: ["Vibrant", "Vibes"],
  W: ["Wild", "Waves"],
  X: ["Xtra", "Xperiences"],
  Y: ["Yearning", "Yells"],
  Z: ["Zany", "Zones"],
}

function getAlliteration(name: string): [string, string] {
  const letter = name.trim()[0]?.toUpperCase() ?? "M"
  return ALLITERATIONS[letter] ?? ["Musical", "Machinations"]

// Prefetch image using link preload that works with Next.js image optimization
function prefetchImage(url: string) {
  const link = document.createElement('link')
  link.rel = 'preload'
  link.as = 'image'
  link.href = url
  link.imageSrcset = `/_next/image?url=${encodeURIComponent(url)}&w=640&q=75 640w, /_next/image?url=${encodeURIComponent(url)}&w=750&q=75 750w`
  document.head.appendChild(link)
}

function FloatingElements() {
  return (
    <div className="fixed inset-0 overflow-hidden pointer-events-none">
      {/* Musical note shapes */}
      {[
        { x: "15%", y: "35%", size: 20, duration: 15 },
        { x: "80%", y: "30%", size: 25, duration: 17 },
        { x: "60%", y: "85%", size: 18, duration: 14 },
        { x: "30%", y: "65%", size: 22, duration: 16 },
      ].map((note, i) => (
        <motion.div
          key={`note-${i}`}
          className="absolute text-primary/10 text-4xl"
          style={{ left: note.x, top: note.y, fontSize: note.size }}
          animate={{
            y: [0, -40, 0],
            opacity: [1.0, 0.2, 1.0],
            rotate: [0, 10, -10, 0],
          }}
          transition={{
            duration: note.duration,
            repeat: Number.POSITIVE_INFINITY,
            ease: "easeInOut",
          }}
        >
          ♪
        </motion.div>
      ))}
    </div>
  )
}

export function SongGame() {
  const [leftSong, setLeftSong] = useState<TrackInfo | null>(null)
  const [rightSong, setRightSong] = useState<TrackInfo | null>(null)
  const [gameState, setGameState] = useState<"playing" | "revealing" | "transitioning" | "loading">("loading")
  const [selectedSide, setSelectedSide] = useState<"left" | "right" | null>(null)
  const [isCorrect, setIsCorrect] = useState<boolean | null>(null)
  const [score, setScore] = useState(0)
  const [highScore, setHighScore] = useState(0)
  const [lives, setLives] = useState(3)
  const [showLeftStreams, setShowLeftStreams] = useState(false)
  const [showRightStreams, setShowRightStreams] = useState(false)
  const [currentAnimation, setCurrentAnimation] = useState<AnimationType>("fling")
  const [isTransitioning, setIsTransitioning] = useState(false)
  const [transitioningSong, setTransitioningSong] = useState<TrackInfo | null>(null)
  const [upcomingSong, setUpcomingSong] = useState<TrackInfo | null>(null)
  const [hasTransitioned, setHasTransitioned] = useState(false)
  const [users, setUsers] = useState<string[]>([])
  const [selectedUser, setSelectedUser] = useState<string | null>(null)
  const [resetCount, setResetCount] = useState(0)

  // Fetch users on mount
  useEffect(() => {
    async function loadUsers() {
      try {
        const { data } = await fetchClient.GET("/users")
        if (data && data.length > 0) {
          setUsers(data)
          setSelectedUser(data[0])
        }
      } catch (err) {
        console.error("Error loading users:", err)
      }
    }
    loadUsers()
  }, [])

  // Fetch two random tracks when selectedUser changes
  useEffect(() => {
    if (!selectedUser) return
    let cancelled = false
    async function loadInitialTracks() {
      try {
        const { data, error } = await fetchClient.GET("/tracks/random/two", {
          params: { query: { user: selectedUser! } },
        })
        if (cancelled) return
        if (data) {
          setLeftSong(data.track1)
          setRightSong(data.track2)
          setGameState("playing")
          setScore(0)
          setLives(3)
          setShowLeftStreams(false)
          setShowRightStreams(false)
          setSelectedSide(null)
          setIsCorrect(null)
          setIsTransitioning(false)
          setHasTransitioned(false)
        } else {
          console.error("Failed to load tracks: " + error)
        }
      } catch (err) {
        console.error("Error loading initial tracks:", err)
      }
    }
    loadInitialTracks()
    return () => { cancelled = true }
  }, [selectedUser])

  // Function to fetch a new random track
  const fetchNewTrack = useCallback(async (): Promise<TrackInfo | null> => {
    try {
      const { data } = await fetchClient.GET("/tracks/random", {
        params: { query: { user: selectedUser ?? undefined } },
      })
      if (data) {
        return data
      } else {
        console.error("Failed to fetch new track")
        return null
      }
    } catch (err) {
      console.error("Error fetching new track:", err)
      return null
    }
  }, [selectedUser])

  const handleSelect = useCallback(
    async (side: "left" | "right") => {
      if (gameState !== "playing" || !leftSong || !rightSong) return

      setSelectedSide(side)
      const selectedSong = side === "left" ? leftSong : rightSong
      const otherSong = side === "left" ? rightSong : leftSong
      const correct = selectedSong.times_played >= otherSong.times_played

      setIsCorrect(correct)
      setGameState("revealing")

      // Fetch new song(s) immediately, before showing streams
      let newSongPromise: Promise<TrackInfo | null> | null = null
      let newTwoSongsPromise: Promise<{ data?: { track1: TrackInfo; track2: TrackInfo } | undefined; error?: any }> | null = null

      if (correct || lives > 1) {
        // Fetch single new track for transition
        newSongPromise = fetchNewTrack().then((track) => {
          if (track?.album_image_url) {
            prefetchImage(track.album_image_url)
          }
          return track
        })
      } else {
        // Game over - fetch two new tracks
        newTwoSongsPromise = fetchClient.GET("/tracks/random/two").then((result) => {
          if (result.data?.track1.album_image_url) {
            prefetchImage(result.data.track1.album_image_url)
          }
          if (result.data?.track2.album_image_url) {
            prefetchImage(result.data.track2.album_image_url)
          }
          return result
        })
      }

      if (side === "left") {
        setShowLeftStreams(true)
        setTimeout(() => setShowRightStreams(true), 800)
      } else {
        setShowRightStreams(true)
        setTimeout(() => setShowLeftStreams(true), 800)
      }

      setTimeout(async () => {
        if (correct || lives > 1) {
          if (!correct) {
            setLives(prev => prev - 1)
          } else {
            setScore((prev) => prev + 1)
          }
          setGameState("transitioning")

          const nextAnimation = getRandomAnimation()
          setCurrentAnimation(nextAnimation)

          const newSong = await newSongPromise
          if (newSong) {
            setTransitioningSong(rightSong)
            setUpcomingSong(newSong)
            setIsTransitioning(true)

            const animDuration = (dramaticAnimations[nextAnimation].transition as { duration: number }).duration * 1000
            setTimeout(() => {
              setLeftSong(rightSong)
              setRightSong(newSong)
              setTransitioningSong(null)
              setUpcomingSong(null)
              setShowLeftStreams(false)
              setShowRightStreams(false)
              setSelectedSide(null)
              setIsCorrect(null)
              setIsTransitioning(false)
              setHasTransitioned(true)
              setGameState("playing")
            }, animDuration + 100)
          }
        } else {
          // Game over - reset everything
          setGameState("transitioning")


          setTimeout(() => {
            setHighScore((prev) => Math.max(prev, score))
            setScore(0)
            setLives(3)
            setResetCount((prev) => prev + 1)
            setSelectedSide(null)
            setIsCorrect(null)
            setShowLeftStreams(false)
            setShowRightStreams(false)
            setIsTransitioning(false)
            setHasTransitioned(false)

            const twoTracksResult = await newTwoSongsPromise

            // Use already fetched tracks
            if (twoTracksResult?.data) {
              setLeftSong(twoTracksResult.data.track1)
              setRightSong(twoTracksResult.data.track2)
              setGameState("playing")
            } else {
              console.error("Failed to load tracks: " + twoTracksResult?.error)
            }
          }, 1000)
        }
      }, 3000)
    },
    [gameState, leftSong, rightSong, lives, score, selectedUser, fetchNewTrack],
  )

  const transitionAnimation = dramaticAnimations[currentAnimation]

  if (gameState === "loading" || !leftSong || !rightSong) {
    return (
      <>
        <div className="w-full max-w-5xl relative z-10 flex items-center justify-center min-h-[500px]">
          <div className="text-center">
            <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-primary mx-auto mb-4"></div>
            <p className="text-muted-foreground">Loading tracks...</p>
          </div>
        </div>
      </>
    )
  }

  const [adj, noun] = getAlliteration(selectedUser ?? "M")

  return (
    <>
      <div className="w-full max-w-5xl relative z-10">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-4xl md:text-5xl font-bold tracking-tight mb-2 text-balance flex flex-wrap items-center justify-center gap-x-3">
            {users.length > 1 ? (
              <Select value={selectedUser ?? ""} onValueChange={(v) => setSelectedUser(v)}>
                <SelectTrigger className="inline-flex w-auto text-4xl md:text-5xl font-bold border-0 border-b-2 border-primary rounded-none bg-transparent px-1 h-auto focus:ring-0 cursor-pointer">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {users.map((u) => (
                    <SelectItem key={u} value={u}>{u}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <span>{selectedUser ?? "..."}</span>
            )}
            <span>&#39;s</span>
            <span>{adj}</span>
            <span>{noun}</span>
          </h1>
          <div className="flex justify-center gap-8 mt-4">
            <div className="text-muted-foreground">
              Score: <span className="text-primary font-bold text-xl">{score}</span>
            </div>
            <div className="text-muted-foreground">
              High Score: <span className="text-accent font-bold text-xl">{highScore}</span>
            </div>
            {/* The key is needed to force rerender when the game restarts*/}
            <div className="flex items-center gap-1" key={lives===3 ? "l": "k"}>
              {[...Array(3)].map((_, i) => (
                <motion.span
                  key={`${resetCount}-${i}`}
                  className={`text-2xl ${i < lives ? "text-red-500" : "text-muted-foreground/30"}`}
                  animate={
                    i === lives && lives < 3
                      ? {
                          scale: [1, 1.5, 0],
                          opacity: [1, 1, 0],
                          rotate: [0, 0, 90],
                        }
                      : {}
                  }
                  transition={{ duration: 0.5 }}
                >
                  {i < lives ? "❤️" : "🖤"}
                </motion.span>
              ))}
            </div>
          </div>
        </div>

        {/* Game Cards */}
        <div className="flex flex-col md:flex-row gap-6 items-center justify-center" style={{ perspective: "1000px" }}>
          <div className="relative" style={{ width: 288 }}>
            {!isTransitioning && (
              <motion.div
                key={`left-${leftSong.spotify_url}-${leftSong.title}-${hasTransitioned}`}
                initial={hasTransitioned ? false : { opacity: 0, x: -50 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.4 }}
              >
                <SongCard
                  song={leftSong}
                  onClick={() => handleSelect("left")}
                  disabled={gameState !== "playing"}
                  showStreams={showLeftStreams}
                  isSelected={selectedSide === "left"}
                  isCorrect={selectedSide === "left" ? isCorrect : null}
                  isWinner={showLeftStreams && leftSong.times_played >= rightSong.times_played}
                />
              </motion.div>
            )}
          </div>

          {/* VS Divider */}
          <div className="flex items-center justify-center">
            <motion.div
              className="w-16 h-16 rounded-full bg-secondary border-2 border-border flex items-center justify-center"
              animate={isTransitioning ? { scale: [1, 0.8, 1], opacity: [1, 0.5, 1] } : {}}
              transition={{ duration: 0.5 }}
            >
              <span className="text-xl font-bold text-muted-foreground">VS</span>
            </motion.div>
          </div>

          <div className="relative" style={{ width: 288 }}>
            {isTransitioning && transitioningSong && (
              <motion.div
                className="absolute top-0 left-0 z-20"
                style={{ transformStyle: "preserve-3d" }}
                animate={transitionAnimation.keyframes}
                transition={transitionAnimation.transition}
              >
                <SongCard
                  song={transitioningSong}
                  onClick={() => {}}
                  disabled={true}
                  showStreams={false}
                  isSelected={false}
                  isCorrect={null}
                  isWinner={false}
                />
              </motion.div>
            )}

            {isTransitioning && upcomingSong ? (
              <motion.div
                initial={{ opacity: 0, x: 100 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.5, delay: 0.4 }}
              >
                <SongCard
                  song={upcomingSong}
                  onClick={() => {}}
                  disabled={true}
                  showStreams={false}
                  isSelected={false}
                  isCorrect={null}
                  isWinner={false}
                />
              </motion.div>
            ) : !isTransitioning ? (
              <motion.div
                key={`left-${rightSong.spotify_url}-${rightSong.title}-${hasTransitioned}`}
                initial={hasTransitioned ? false : { opacity: 0, x: 50 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.4 }}
              >
                <SongCard
                  song={rightSong}
                  onClick={() => handleSelect("right")}
                  disabled={gameState !== "playing"}
                  showStreams={showRightStreams}
                  isSelected={selectedSide === "right"}
                  isCorrect={selectedSide === "right" ? isCorrect : null}
                  isWinner={showRightStreams && rightSong.times_played >= leftSong.times_played}
                />
              </motion.div>
            ) : null}
          </div>
        </div>
      </div>
    </>
  )
}
