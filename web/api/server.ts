import { useApiFetch } from "./customFetch";
import type { ServerInfo, ToSetup } from "./model";

export async function setup(body: ToSetup) {
    return useApiFetch(`/setup`, {
        watch: false,
        method: 'PUT',
        body
    })
}

export async function getServerInfo() {
    return useApiFetch<ServerInfo>(`/server_info`)
}

export async function reloadMedias() {
    return useApiFetch(`/medias/reload`, {
        watch: false,
        method: 'PUT'
    })
}

export async function reloadPlugins() {
    return useApiFetch(`/plugins/reload`, {
        watch: false,
        method: 'PUT'
    })
}