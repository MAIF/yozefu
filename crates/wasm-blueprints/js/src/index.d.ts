declare module 'main' {
    export function matches(): I32;
    export function parse_parameters(): I32;
}

declare interface KafkaRecord {
    value: string;
    key: string;
    topic: string;
    timestamp: number;
    partition: number;
    offset: number;
    headers: record<string, string>;
}

declare interface FilterInput {
    record: KafkaRecord;
    params: FilterParams;
}

declare interface FilterResult {
    match: boolean;
}