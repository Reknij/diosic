import { useApiFetch } from "./customFetch";
import type { PubMediaInfo, GetMediasQuery, ListSlice } from "./model";

export function getMediaFileAddress(id: number): string {
    const auth = useCookie(AUTH_COOKIE_NAME).value;
    const config = useRuntimeConfig();  
    const origin = import.meta.dev? config.public.devBaseUrl : config.public.baseUrl;
    return `${origin}/api/media_file/${id}?auth=${auth}`;
}

export async function getMedias(query: GetMediasQuery) {
    return useApiFetch<ListSlice<PubMediaInfo>>('/medias', {
        query
    })
}

export async function getMedia(id: number) {
    return useApiFetch<PubMediaInfo>(`/media_info/${id}`)
}