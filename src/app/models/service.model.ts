export enum TotpAlgorithm {
    SHA1,
    SHA256,
    SHA512
}

export interface Service {
    id: string;
    issuer: string;
    secret: string;
    name: string;
    algorithm: TotpAlgorithm; // or use enum if you prefer
    digits: number;
    period: number;
    icon?: string;
} 
