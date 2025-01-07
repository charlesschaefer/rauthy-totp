export interface Service {
    id: string;
    issuer: string;
    secret: string;
    name: string;
    algorithm: string; // or use enum if you prefer
    digits: number;
    period: number;
} 
