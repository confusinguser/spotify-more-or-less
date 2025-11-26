import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import { components, paths } from "./schema";

export const fetchClient = createFetchClient<paths>({
  baseUrl: "/api",
});
export const $api = createClient(fetchClient);

export type TrackInfo = components["schemas"]["TrackInfo"]
