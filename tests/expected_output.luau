--- A complex schema with various data types and structures for testing
export type Root = {
    --- Required when active is true
    --- @format date-time
    activationDate: string?,
    active: boolean,
    allOfExample: ({ name: string } & { code: string })?,
    anyOfExample: (string | number | boolean)?,
    billingAddress: Address?,
    --- Simplified score enum
    compactScore: number?,
    --- Simplified status enum
    compactStatus: "active" | "paused" | "deleted"?,
    --- @minItems 1
    --- @maxItems 5
    --- @uniqueItems true
    compactTags: { string }?,
    complexArray: { (string | number | boolean | { type: string?, value: any? }) }?,
    --- @format date-time
    createdAt: string,
    --- Required when active is false
    --- @maxLength 200
    deactivationReason: string?,
    --- @maxLength 500
    description: string?,
    dimensions: Dimensions?,
    enhancedDimensions: EnhancedDimensions?,
    enhancedMeta: ({ author: string, description: string?, version: string } & Timestamps)?,
    exclusiveChoice: ("a" | "b" | "c")?,
    --- @uniqueItems true
    flags: { "featured" | "verified" | "premium" | "legacy" }?,
    flexiblePayload: (string | number | ExtraPayload)?,
    --- Unique identifier for the entity
    --- @format uuid
    id: string,
    --- Flexible key-value store for additional data
    metadata: { [string]: any }?,
    --- @minLength 1
    --- @maxLength 100
    --- @pattern ^[a-zA-Z0-9 _-]+$
    name: string,
    --- String field that can be explicitly set to null
    nullableField: (string | nil)?,
    --- @maxProperties 10
    --- @minimum 0
    numericConfig: { [string]: number }?,
    --- Alternative numeric identifier
    --- @minimum 1
    --- @maximum 999
    numericId: number?,
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
    --- Required when status is suspended
    --- @minLength 1
    --- @maxLength 200
    suspensionReason: string?,
    --- @maxItems 50
    --- @uniqueItems true
    tags: { string }?,
    --- @format date-time
    timestamps: { [string]: string }?,
    --- @format date-time
    updatedAt: string?,
    --- @pattern ^\d+\.\d+\.\d+$
    version: string,
}

--- Address information
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

--- Dimensions
export type Dimensions = {
    --- @minimum 0
    depth: number?,
    --- @minimum 0
    height: number?,
    unit: "cm" | "m" | "in" | "ft"?,
    --- @minimum 0
    width: number?,
}

--- Enhanced Dimensions
--- Intersection type (all conditions must be met)
export type EnhancedDimensions = Dimensions & { weight: number?, weightUnit: "kg" | "lb" | "g"? }

--- Extra payload
export type ExtraPayload = {
    data: any?,
    kind: "analytics" | "metrics" | "events" | "logs",
    --- @multipleOf 0.5
    value: number?,
}

--- Geographic coordinates
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

--- Person information
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

--- Product information
export type Product = {
    --- @minItems 1
    categories: { "electronics" | "clothing" | "books" | "home" | "sports" }?,
    currency: "USD" | "EUR" | "GBP" | "JPY" | "CAD"?,
    --- @maxLength 1000
    description: string?,
    --- @pattern ^[A-Z0-9]{8}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{12}$
    id: string,
    inStock: boolean?,
    metadata: { [string]: string }?,
    --- @minLength 1
    --- @maxLength 100
    name: string,
    --- @exclusiveMinimum 0
    price: number,
}

--- Timestamps
export type Timestamps = {
    --- @format date-time
    createdAt: string?,
    --- @format date-time
    updatedAt: string?,
}
