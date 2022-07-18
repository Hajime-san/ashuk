export interface SerializedErrorBaseObject {
    message?: string
    status?: number
}

export interface SerializedErrorObject extends SerializedErrorBaseObject {
    name: string
}
