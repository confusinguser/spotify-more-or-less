"use client"

import { useEffect, useState } from "react"
import { motion, AnimatePresence } from "framer-motion"
import Image from "next/image"

interface Song {
  id: number
  title: string
  artist: string
  streams: number
  albumArt: string
}

interface SongCardProps {
  song: Song
  onClick: () => void
  disabled: boolean
  showStreams: boolean
  isSelected: boolean
  isCorrect: boolean | null
  isWinner: boolean
}

function AnimatedCounter({ target, duration = 1500 }: { target: number; duration?: number }) {
  const [count, setCount] = useState(0)

  useEffect(() => {
    const startTime = Date.now()
    const animate = () => {
      const elapsed = Date.now() - startTime
      const progress = Math.min(elapsed / duration, 1)
      // Easing function for smooth deceleration
      const eased = 1 - Math.pow(1 - progress, 3)
      setCount(Math.floor(target * eased))

      if (progress < 1) {
        requestAnimationFrame(animate)
      }
    }
    animate()
  }, [target, duration])

  return <span className="font-mono">{count.toLocaleString()}</span>
}

export function SongCard({ song, onClick, disabled, showStreams, isSelected, isCorrect, isWinner }: SongCardProps) {
  const getBorderColor = () => {
    if (!showStreams) return "border-border"
    if (isWinner) return "border-primary"
    return "border-destructive/50"
  }

  const getGlowColor = () => {
    if (!showStreams) return ""
    if (isWinner) return "shadow-[0_0_30px_rgba(134,239,172,0.3)]"
    return "shadow-[0_0_30px_rgba(239,68,68,0.2)]"
  }

  return (
    <motion.button
      onClick={onClick}
      disabled={disabled}
      whileHover={disabled ? {} : { scale: 1.02 }}
      whileTap={disabled ? {} : { scale: 0.98 }}
      className={`
        relative w-72 bg-card rounded-xl border-2 overflow-hidden transition-all duration-300
        ${getBorderColor()} ${getGlowColor()}
        ${disabled ? "cursor-default" : "cursor-pointer hover:border-primary/50"}
      `}
    >
      {/* Album Art */}
      <div className="relative aspect-square overflow-hidden">
        <Image
          src={song.albumArt || "/placeholder.svg"}
          alt={`${song.title} album art`}
          fill
          className="object-cover"
        />

        {/* Selection Overlay */}
        <AnimatePresence>
          {isSelected && isCorrect !== null && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className={`
                absolute inset-0 flex items-center justify-center
                ${isCorrect ? "bg-primary/20" : "bg-destructive/20"}
              `}
            >
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring", stiffness: 300, damping: 20 }}
                className={`
                  w-20 h-20 rounded-full flex items-center justify-center
                  ${isCorrect ? "bg-primary text-primary-foreground" : "bg-destructive text-destructive-foreground"}
                `}
              >
                <span className="text-4xl">{isCorrect ? "✓" : "✗"}</span>
              </motion.div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Song Info */}
      <div className="p-4">
        <h3 className="font-bold text-lg text-card-foreground truncate">{song.title}</h3>
        <p className="text-muted-foreground text-sm truncate">{song.artist}</p>
      </div>

      {/* Stream Counter Triangle */}
      <AnimatePresence>
        {showStreams && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
            className="relative"
          >
            {/* Triangle pointer */}
            <div className="flex justify-center -mb-px">
              <div
                className={`
                  w-0 h-0 border-l-8 border-r-8 border-b-8 border-transparent
                  ${isWinner ? "border-b-primary" : "border-b-secondary"}
                `}
              />
            </div>

            {/* Counter box */}
            <motion.div
              initial={{ scaleY: 0 }}
              animate={{ scaleY: 1 }}
              transition={{ duration: 0.2, delay: 0.1 }}
              className={`
                py-3 px-4 text-center origin-top
                ${isWinner ? "bg-primary text-primary-foreground" : "bg-secondary text-secondary-foreground"}
              `}
            >
              <p className="text-xs uppercase tracking-wider mb-1 opacity-80">Streams</p>
              <p className="text-xl font-bold">
                <AnimatedCounter target={song.streams} />
              </p>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.button>
  )
}
