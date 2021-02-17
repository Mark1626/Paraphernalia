// @ts-check
const crypto = require("crypto")

const debug_enabled = process.env.DEBUG || false

/**
 version:claimedBits:timestamp:resource:extension:randomValue:counter
 **/
const version = 1

const getTimestamp = () => {
  const date = new Date()
  const year = date.getFullYear() % 100
  const month = (date.getMonth() + 1)
  const monthStr = month < 10 ? `0${month}` : `${month}`
  const day = date.getDate()
  const dayStr = day < 10 ? `0${day}` : `${day}`
  return `${year}${monthStr}${dayStr}`
}

const mintCash = (resource, claimedBits) => {
  const timestamp = getTimestamp()
  const extension = ""
  // const hash = crypto.createHash('sha1')
  let prefix = `${version}:${claimedBits}:${timestamp}:${resource}:${extension}`

  const random = crypto.randomBytes(8)
  let counter = BigInt(`0x${crypto.randomBytes(8).toString('hex')}`)

  prefix = `${prefix}:${random.toString('hex')}`
  let stamp = `${prefix}:${counter.toString(16)}`

  let currentClaimedBits = 0
  let iterations = 0

  for (; currentClaimedBits < claimedBits; ) {
    const hash = crypto.createHash('sha1')
    counter++
    stamp = `${prefix}:${counter.toString(16)}`
    hash.update(stamp)
    const messageSHA = hash.digest()
    currentClaimedBits = getNumberOfLeadingZeros(messageSHA)
    
    if (debug_enabled) {
      console.log(stamp, currentClaimedBits, messageSHA)
      if (iterations > 10_000_000) break
      iterations ++
    }
  }

  return stamp
}

/**
 * 
 * @param {Buffer} values 
 */
const getNumberOfLeadingZeros = (values) => {
  const bytes = values.valueOf()
  let result = 0
  for (let i = 0; i < bytes.length; i++) {
    const zeros = getNumberOfLeadingZerosInByte(bytes[i])
    result += zeros
    if (zeros != 8) break
  }
  return result
}

/**
 * 
 * @param {number} byte 
 */
const getNumberOfLeadingZerosInByte = (byte) => {
  if (byte < 0) return 0
  if (byte < 1<<0) return 8
  if (byte < 1<<1) return 7
  if (byte < 1<<2) return 6
  if (byte < 1<<3) return 5
  if (byte < 1<<4) return 4
  if (byte < 1<<5) return 3
  if (byte < 1<<6) return 2
  if (byte < 1<<7) return 1
  return 0
}

const computeSHA = (content) => {
  const hash = crypto.createHash('sha1')
  hash.update(content)
  return hash.copy().digest()
}

const fetchEstimates = (iterations = 20) => {
  mintCash('test', 16)
  let duration = Date.now()
  for (let i = 0; i < iterations; i++) {
    mintCash('test', 16)
  }
  duration = Date.now() - duration
  return duration / iterations
}

const estimateTime = (val) => fetchEstimates() * Math.pow(2, val - 16)

module.exports = {
  mintCash,
  estimateTime,
}
