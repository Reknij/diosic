import { useApiFetch } from "./customFetch"
import type { GetSourcesQuery, ListSlice, SourceInfo } from "./model"


export async function getSources(query: GetSourcesQuery) {
    return useApiFetch<ListSlice<SourceInfo>>(`/sources`, {
        query
    })
}