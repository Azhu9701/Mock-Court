import type { Metadata, Viewport } from "next";
import { Providers } from "@/components/providers";
import { ShellLayout } from "@/components/shell-layout";
import { BreadcrumbProvider } from "@/contexts/breadcrumb-context";
import "./globals.css";

export const metadata: Metadata = {
  title: "模拟仲裁庭",
  description: "AI 模拟仲裁庭 — 劳动争议多角色并行庭审系统",
  manifest: "/manifest.json",
  appleWebApp: {
    capable: true,
    statusBarStyle: "black-translucent",
    title: "模拟仲裁庭",
  },
  icons: {
    icon: "/icon.svg",
    apple: "/apple-icon.svg",
  },
};

export const viewport: Viewport = {
  themeColor: "#0a0a0a",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <head>
        <meta name="mobile-web-app-capable" content="yes" />
      </head>
      <body className="antialiased">
        <script src="/sw-register.js" defer />
        <Providers>
          <BreadcrumbProvider>
            <ShellLayout>{children}</ShellLayout>
          </BreadcrumbProvider>
        </Providers>
      </body>
    </html>
  );
}
