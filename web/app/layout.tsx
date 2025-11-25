import type React from "react"
import type { Metadata } from "next"
import "./globals.css"

export const metadata: Metadata = {
  title: "Spotify More/Less",
  description: "Nice game :)",
}

export default function RootLayout({
                                     children,
                                   }: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
    <body className={`font-sans antialiased`}>
    {children}
    </body>
    </html>
  )
}
