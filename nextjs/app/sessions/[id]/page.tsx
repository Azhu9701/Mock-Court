import SessionDetailView from "@/components/session-detail-view";

export const dynamic = "force-dynamic";

export default async function SessionPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <SessionDetailView id={id} />;
}
