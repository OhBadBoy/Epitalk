import type { FetchClient } from "./fetchClient";
import type { Server, Channel, Member, Invite, Ban, BanMemberRequest } from "./schemas/servers.schema";

type PaginatedResponse<T> = {
  data: T[];
  total: number;
  limit: number;
  offset: number;
};

function toList<T>(payload: T[] | PaginatedResponse<T> | null | undefined): T[] {
  if (Array.isArray(payload)) return payload;
  if (payload && Array.isArray((payload as PaginatedResponse<T>).data)) {
    return (payload as PaginatedResponse<T>).data;
  }
  return [];
}

export function createServersApi(client: FetchClient) {
  return {
    // Servers
    list: async () => {
      const payload = await client.get<Server[] | PaginatedResponse<Server>>("/servers");
      return toList(payload);
    },
    get: (serverId: string) => client.get<Server>(`/servers/${serverId}`),
    create: (name: string) => client.post<Server>("/servers", { name }),
    update: (serverId: string, name: string) =>
      client.patch<Server>(`/servers/${serverId}`, { name }),
    delete: (serverId: string) => client.delete<void>(`/servers/${serverId}`),
    leave: (serverId: string) => client.post<void>(`/servers/${serverId}/leave`),
    transfer: (serverId: string, newOwnerId: string) =>
      client.post<void>(`/servers/${serverId}/transfer`, { new_owner_id: newOwnerId }),

    // Channels (nested under servers)
    // Backend expects { name, kind } where kind is "Text" (PascalCase)
    listChannels: async (serverId: string) => {
      const payload = await client.get<Channel[] | PaginatedResponse<Channel>>(
        `/servers/${serverId}/channels`,
      );
      return toList(payload);
    },
    getChannel: (serverId: string, channelId: string) =>
      client.get<Channel>(`/servers/${serverId}/channels/${channelId}`),
    createChannel: (serverId: string, name: string, kind: "Text" = "Text") =>
      client.post<Channel>(`/servers/${serverId}/channels`, { name, kind }),
    updateChannel: (serverId: string, channelId: string, name: string) =>
      client.patch<Channel>(`/servers/${serverId}/channels/${channelId}`, { name }),
    deleteChannel: (serverId: string, channelId: string) =>
      client.delete<void>(`/servers/${serverId}/channels/${channelId}`),

    // Members (nested under servers)
    listMembers: async (serverId: string) => {
      const payload = await client.get<Member[] | PaginatedResponse<Member>>(
        `/servers/${serverId}/members`,
      );
      return toList(payload);
    },
    getMember: (serverId: string, memberId: string) =>
      client.get<Member>(`/servers/${serverId}/members/${memberId}`),
    updateMemberRole: (serverId: string, memberId: string, role: string) =>
      client.patch<Member>(`/servers/${serverId}/members/${memberId}/role`, { role }),
    kickMember: (serverId: string, memberId: string) =>
      client.delete<void>(`/servers/${serverId}/members/${memberId}`),
        banMember: (
      serverId: string,
      memberId: string,
      payload?: BanMemberRequest
    ) =>
      client.post<Ban>(`/servers/${serverId}/members/${memberId}/ban`, {
        reason: payload?.reason ?? null,
        expires_at: payload?.expires_at ?? null,
      }),
    unbanMember: (serverId: string, memberId: string) =>
      client.delete<{ unbanned: boolean }>(
        `/servers/${serverId}/members/${memberId}/ban`
      ),
    listBans: (serverId: string) =>
      client.get<Ban[]>(`/servers/${serverId}/members/bans`),

    // Invites
    createInvite: (serverId: string, expiresInHours?: number, maxUses?: number) =>
      client.post<Invite>(`/servers/${serverId}/invites`, { 
        expires_in_hours: expiresInHours, 
        max_uses: maxUses 
      }),
    listInvites: (serverId: string) =>
      client.get<Invite[]>(`/servers/${serverId}/invites`),
    listActiveInvites: (serverId: string) =>
      client.get<Invite[]>(`/servers/${serverId}/invites/active`),
    deleteInvite: (serverId: string, inviteCode: string) =>
      client.delete<void>(`/servers/${serverId}/invites/${inviteCode}`),
    
    // Join server via invite code (top-level route)
    joinByInvite: (code: string) =>
      client.post<Server>("/join", { code }),
  };
}
