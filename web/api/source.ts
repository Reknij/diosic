import { useApiFetch } from "./customFetch"
import type { GetSourcesQuery, ListSlice, SourceInfo } from "./model"


export async function getSources(source: string, query: GetSourcesQuery) {
    return useApiFetch<ListSlice<SourceInfo>>(`/sources/${source}`, {
        query
    })
}

export async function getSource(title: string) {
    return useApiFetch<SourceInfo>(`/source/${title}`)
}