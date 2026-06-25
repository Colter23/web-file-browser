import type {AuditCleanupResponse, HealthResponse, MetricsResponse, ReadinessResponse} from "../class";
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
    const response = await network.get("/api/ready", {
        validateStatus: status => (status >= 200 && status < 300) || status === 503
    });
    return response.data;
}

export const cleanupAudit = async (): Promise<AuditCleanupResponse> => {
    const response = await network.post("/api/audit/cleanup");
    return response.data;
}
