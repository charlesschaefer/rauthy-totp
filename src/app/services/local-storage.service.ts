import { Injectable } from '@angular/core';

@Injectable({
    providedIn: 'root',
})
export class LocalStorageService {
    constructor() { }

    // Set a value in local storage
    setItem(key: string, value: string): void {
        localStorage.setItem(key, value);
    }

    // Get a value from local storage
    getItem(key: string): string | null {
        return localStorage.getItem(key);
    }

    // Remove a value from local storage
    removeItem(key: string): void {
        localStorage.removeItem(key);
    }

    hasItem(key: string): boolean {
        const item = localStorage.getItem(key);
        return (item && item != "") as boolean;
    }

    // Clear all items from local storage
    clear(): void {
        localStorage.clear();
    }
}
