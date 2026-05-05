import InviteJoinClient from "./invite-join-client";

type PageProps = {
  params: Promise<{ code: string }>;
};

export default async function InvitePage({ params }: PageProps) {
  const { code } = await params;
  return <InviteJoinClient code={code} />;
}
