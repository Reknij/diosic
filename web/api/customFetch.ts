type useFetchType = typeof useFetch;

export const useApiFetch: useFetchType = (path, opts = {}) => {
    const auth = useCookie(AUTH_COOKIE_NAME);
    const config = useRuntimeConfig();
    const headers: any = {}
    if (auth.value) {
        headers["X-Authorization"] = auth.value.toString();
    }
    opts.baseURL = `${import.meta.dev ? config.public.devBaseUrl : config.public.baseUrl}/api`;
    opts.headers = headers;
    opts.key = `${path}${JSON.stringify(opts.query)}`;
    // opts.credentials = 'include'; // cloudflare workers will broken.
    return useFetch(path, opts);
};