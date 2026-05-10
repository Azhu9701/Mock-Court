import type { Metadata } from "next";
import { Providers } from "@/components/providers";
import { ShellLayout } from "@/components/shell-layout";
import "./globals.css";

export const metadata: Metadata = {
  title: "万民幡",
  description: "24 位思想家的 AI 对话实验室",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <body className="antialiased">
        <Providers>
          <ShellLayout>{children}</ShellLayout>
        </Providers>
      </body>
    </html>
  );
}
