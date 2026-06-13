import type {HealthResponse, MetricsResponse, ReadinessResponse} from "../class";
import network from "../network";

export const getMetrics = async (): Promise<MetricsResponse> => {
    const response = await network.get("/api/metrics");
    return response.data;
}

export const getHealth = async (): Promise<HealthResponse> => {
    const response = await network.get("/api/health");
    return response.data;
}

export const getReadiness = async (): Promise<ReadinessResponse> => {
    const response = await network.get("/api/ready");
    return response.data;
}
