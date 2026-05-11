import type { Metadata, Viewport } from "next";
import { Providers } from "@/components/providers";
import { ShellLayout } from "@/components/shell-layout";
import { BreadcrumbProvider } from "@/contexts/breadcrumb-context";
import "./globals.css";

export const metadata: Metadata = {
  title: "万民幡",
  description: "多 AI 人格并行推理系统 - 24 位思想家的思维碰撞",
  manifest: "/manifest.json",
  appleWebApp: {
    capable: true,
    statusBarStyle: "black-translucent",
    title: "万民幡",
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
        <script
          dangerouslySetInnerHTML={{
            __html: `
              if ('serviceWorker' in navigator) {
                window.addEventListener('load', () => {
                  navigator.serviceWorker.register('/sw.js').catch(() => {});
                });
              }
            `,
          }}
        />
        <Providers>
          <BreadcrumbProvider>
            <ShellLayout>{children}</ShellLayout>
          </BreadcrumbProvider>
        </Providers>
      </body>
    </html>
  );
}
