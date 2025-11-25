"use client"

import { useState, useCallback } from "react"
import { SongCard } from "./song-card"
import { motion } from "framer-motion"

interface Song {
  id: number
  title: string
  artist: string
  streams: number
  albumArt: string
}

const allSongs: Song[] = [
  {
    id: 1,
    title: "Blinding Lights",
    artist: "The Weeknd",
    streams: 4200000000,
    albumArt: "/neon-city-lights-album-cover.jpg",
  },
  {
    id: 2,
    title: "Shape of You",
    artist: "Ed Sheeran",
    streams: 3800000000,
    albumArt: "/orange-geometric-shapes-album-cover.jpg",
  },
  {
    id: 3,
    title: "Dance Monkey",
    artist: "Tones and I",
    streams: 2900000000,
    albumArt: "/colorful-monkey-illustration-album.jpg",
  },
  {
    id: 4,
    title: "Rockstar",
    artist: "Post Malone",
    streams: 2700000000,
    albumArt: "/dark-rock-star-aesthetic-album.jpg",
  },
  {
    id: 5,
    title: "One Dance",
    artist: "Drake",
    streams: 2500000000,
    albumArt: "/tropical-sunset-vibes-album-cover.jpg",
  },
  {
    id: 6,
    title: "Sunflower",
    artist: "Post Malone",
    streams: 2400000000,
    albumArt: "/yellow-sunflower-artistic-album.jpg",
  },
  {
    id: 7,
    title: "Closer",
    artist: "The Chainsmokers",
    streams: 2300000000,
    albumArt: "/electronic-blue-waves-album.jpg",
  },
  {
    id: 8,
    title: "Starboy",
    artist: "The Weeknd",
    streams: 2200000000,
    albumArt: "/futuristic-star-neon-album-cover.jpg",
  },
  {
    id: 9,
    title: "Perfect",
    artist: "Ed Sheeran",
    streams: 2100000000,
    albumArt: "/romantic-soft-pink-album-cover.jpg",
  },
  {
    id: 10,
    title: "Heat Waves",
    artist: "Glass Animals",
    streams: 2000000000,
    albumArt: "/psychedelic-heat-waves-album.jpg",
  },
  {
    id: 11,
    title: "Bad Guy",
    artist: "Billie Eilish",
    streams: 1900000000,
    albumArt: "/edgy-green-black-album-cover.jpg",
  },
  {
    id: 12,
    title: "Lucid Dreams",
    artist: "Juice WRLD",
    streams: 1800000000,
    albumArt: "/dreamy-purple-clouds-album.jpg",
  },
]

function getRandomSong(exclude: number[]): Song {
  const available = allSongs.filter((s) => !exclude.includes(s.id))
  return available[Math.floor(Math.random() * available.length)]
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
      x: [0, -100, -280, -4000],
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

function FloatingElements() {
  const elements = [
    { size: 60, x: "10%", y: "20%", duration: 20, delay: 0 },
    { size: 40, x: "85%", y: "15%", duration: 25, delay: 2 },
    { size: 80, x: "70%", y: "70%", duration: 22, delay: 1 },
    { size: 30, x: "20%", y: "80%", duration: 18, delay: 3 },
    { size: 50, x: "50%", y: "10%", duration: 24, delay: 0.5 },
    { size: 35, x: "90%", y: "50%", duration: 21, delay: 1.5 },
    { size: 45, x: "5%", y: "50%", duration: 23, delay: 2.5 },
    { size: 55, x: "40%", y: "85%", duration: 19, delay: 0.8 },
  ]

  return (
    <div className="fixed inset-0 overflow-hidden pointer-events-none">
      {elements.map((el, i) => (
        <motion.div
          key={i}
          className="absolute rounded-full opacity-10"
          style={{
            width: el.size,
            height: el.size,
            left: el.x,
            top: el.y,
            background:
              i % 2 === 0
                ? "linear-gradient(135deg, hsl(var(--primary)) 0%, transparent 100%)"
                : "linear-gradient(135deg, hsl(var(--accent)) 0%, transparent 100%)",
          }}
          animate={{
            y: [0, -30, 0, 30, 0],
            x: [0, 20, 0, -20, 0],
            scale: [1, 1.1, 1, 0.9, 1],
            rotate: [0, 90, 180, 270, 360],
          }}
          transition={{
            duration: el.duration,
            delay: el.delay,
            repeat: Number.POSITIVE_INFINITY,
            ease: "easeInOut",
          }}
        />
      ))}
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
            opacity: [0.1, 0.2, 0.1],
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
  const [leftSong, setLeftSong] = useState<Song>(allSongs[0])
  const [rightSong, setRightSong] = useState<Song>(allSongs[1])
  const [gameState, setGameState] = useState<"playing" | "revealing" | "transitioning">("playing")
  const [selectedSide, setSelectedSide] = useState<"left" | "right" | null>(null)
  const [isCorrect, setIsCorrect] = useState<boolean | null>(null)
  const [score, setScore] = useState(0)
  const [highScore, setHighScore] = useState(0)
  const [showLeftStreams, setShowLeftStreams] = useState(false)
  const [showRightStreams, setShowRightStreams] = useState(false)
  const [usedSongIds, setUsedSongIds] = useState<number[]>([allSongs[0].id, allSongs[1].id])
  const [currentAnimation, setCurrentAnimation] = useState<AnimationType>("fling")
  const [isTransitioning, setIsTransitioning] = useState(false)
  const [transitioningSong, setTransitioningSong] = useState<Song | null>(null)
  const [upcomingSong, setUpcomingSong] = useState<Song | null>(null)
  const [hasTransitioned, setHasTransitioned] = useState(false)

  const handleSelect = useCallback(
    (side: "left" | "right") => {
      if (gameState !== "playing") return

      setSelectedSide(side)
      const selectedSong = side === "left" ? leftSong : rightSong
      const otherSong = side === "left" ? rightSong : leftSong
      const correct = selectedSong.streams >= otherSong.streams

      setIsCorrect(correct)
      setGameState("revealing")

      if (side === "left") {
        setShowLeftStreams(true)
        setTimeout(() => setShowRightStreams(true), 800)
      } else {
        setShowRightStreams(true)
        setTimeout(() => setShowLeftStreams(true), 800)
      }

      setTimeout(() => {
        if (correct) {
          setScore((prev) => prev + 1)
          setGameState("transitioning")

          const nextAnimation = getRandomAnimation()
          setCurrentAnimation(nextAnimation)

          const newSong = getRandomSong([...usedSongIds])
          setUsedSongIds((prev) => [...prev, newSong.id])
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
        } else {
          setTimeout(() => {
            setHighScore((prev) => Math.max(prev, score))
            setScore(0)
            const song1 = getRandomSong([])
            const song2 = getRandomSong([song1.id])
            setUsedSongIds([song1.id, song2.id])
            setLeftSong(song1)
            setRightSong(song2)
            setShowLeftStreams(false)
            setShowRightStreams(false)
            setSelectedSide(null)
            setIsCorrect(null)
            setHasTransitioned(false)
            setGameState("playing")
          }, 1000)
        }
      }, 3000)
    },
    [gameState, leftSong, rightSong, score, usedSongIds],
  )

  const transitionAnimation = dramaticAnimations[currentAnimation]

  return (
    <>
      <FloatingElements />

      <div className="w-full max-w-5xl relative z-10">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-4xl md:text-5xl font-bold tracking-tight mb-2 text-balance">
            Which song has more streams?
          </h1>
          <div className="flex justify-center gap-8 mt-4">
            <div className="text-muted-foreground">
              Score: <span className="text-primary font-bold text-xl">{score}</span>
            </div>
            <div className="text-muted-foreground">
              High Score: <span className="text-accent font-bold text-xl">{highScore}</span>
            </div>
          </div>
        </div>

        {/* Game Cards */}
        <div className="flex flex-col md:flex-row gap-6 items-center justify-center" style={{ perspective: "1000px" }}>
          <div className="relative" style={{ width: 288 }}>
            {!isTransitioning && (
              <motion.div
                key={`left-${leftSong.id}-${hasTransitioned}`}
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
                  isWinner={showLeftStreams && leftSong.streams >= rightSong.streams}
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
                key={`right-${rightSong.id}-${hasTransitioned}`}
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
                  isWinner={showRightStreams && rightSong.streams > leftSong.streams}
                />
              </motion.div>
            ) : null}
          </div>
        </div>
      </div>
    </>
  )
}
