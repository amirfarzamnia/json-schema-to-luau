--- A complex schema with various data types and structures for testing
export type Root = {
    active: boolean,
    allOfExample: ({ name: string } & { code: string })?,
    anyOfExample: (string | number | boolean)?,
    billingAddress: Address?,
    complexArray: { (string | number | boolean | { type: string?, value: any? }) }?,
    conditionalField: any?,
    --- @format date-time
    createdAt: string,
    --- @maxLength 500
    description: string?,
    dimensions: { depth: number?, height: number?, unit: "cm" | "m" | "in" | "ft"?, width: number? }?,
    --- @uniqueItems true
    flags: { "featured" | "verified" | "premium" | "legacy" }?,
    --- Unique identifier for the entity
    --- @format uuid
    id: string,
    --- Flexible key-value store for additional data
    metadata: { [string]: any }?,
    --- @minLength 1
    --- @maxLength 100
    --- @pattern ^[a-zA-Z0-9 _-]+$
    name: string,
    primaryUser: Person,
    priority: number?,
    --- @minItems 1
    --- @maxItems 100
    products: { Product },
    --- @minimum 0
    --- @maximum 100
    --- @multipleOf 0.5
    score: number?,
    --- @minItems 0
    --- @maxItems 10
    secondaryUsers: { Person }?,
    settings: { notifications: { email: boolean?, push: boolean?, sms: boolean? }?, privacy: { level: "public" | "private" | "hidden"?, shareAnalytics: boolean? }? }?,
    shippingAddress: Address?,
    status: "pending" | "active" | "suspended" | "terminated"?,
    --- @maxItems 50
    --- @uniqueItems true
    tags: { string }?,
    timestamps: { [string]: any }?,
    --- @format date-time
    updatedAt: string?,
    --- @pattern ^\d+\.\d+\.\d+$
    version: string,
}

export type Address = {
    --- @minLength 1
    --- @maxLength 50
    city: string,
    coordinates: GeoCoordinates?,
    country: "US" | "CA" | "UK" | "DE" | "FR" | "JP",
    --- @pattern ^[0-9]{5}(-[0-9]{4})?$
    postalCode: string?,
    --- @minLength 2
    --- @maxLength 2
    state: string?,
    --- @minLength 1
    --- @maxLength 100
    street: string,
}

export type GeoCoordinates = {
    --- Elevation in meters
    elevation: number?,
    --- @minimum -90
    --- @maximum 90
    latitude: number,
    --- @minimum -180
    --- @maximum 180
    longitude: number,
}

export type Person = {
    --- @minimum 0
    --- @maximum 150
    age: number,
    --- @format email
    email: string?,
    --- @minLength 1
    --- @maxLength 50
    firstName: string,
    isActive: boolean?,
    --- @minLength 1
    --- @maxLength 50
    lastName: string,
    --- @maxLength 50
    middleName: string?,
    --- @pattern ^\+?[1-9]\d{1,14}$
    phone: string?,
    --- @minItems 0
    --- @maxItems 10
    --- @uniqueItems true
    tags: { string }?,
}

export type Product = {
    --- @minItems 1
    categories: { "electronics" | "clothing" | "books" | "home" | "sports" }?,
    currency: "USD" | "EUR" | "GBP" | "JPY" | "CAD"?,
    --- @maxLength 1000
    description: string?,
    --- @pattern ^[A-Z0-9]{8}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{12}$
    id: string,
    inStock: boolean?,
    metadata: { [string]: any }?,
    --- @minLength 1
    --- @maxLength 100
    name: string,
    --- @minimum 0
    --- @exclusiveMinimum 0
    price: number,
}