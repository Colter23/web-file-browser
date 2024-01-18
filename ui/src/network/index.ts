import axios, {AxiosResponse} from "axios";
import config from "../config";

/**
 * Network Api Access
 * Package of Axios
 * @author zhengyi59
 */
const instance = axios.create({
    baseURL: config.BASE_URL
})

const get = async (url: string): Promise<AxiosResponse> => {
    return await instance.get(url)
}

const post = async (url: string, data: Object = {}): Promise<AxiosResponse> => {
    return await instance.post(url, data)
}

export default {
    instance,
    get,
    post
}