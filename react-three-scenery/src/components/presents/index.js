import { Present } from './Present'
import {
  presentACubeMtl,
  presentACubeObj,
  presentARectMtl,
  presentARectObj,
  presentARoundMtl,
  presentARoundObj,
  presentBCubeMtl,
  presentBCubeObj,
  presentBRectMtl,
  presentBRectObj,
  presentBRoundMtl,
  presentBRoundObj,
} from '../constants'

export function PresentACube(props) {
  return (
    <Present objUrl={presentACubeObj} mtlUrl={presentACubeMtl} {...props} />
  )
}

export function PresentARectangle(props) {
  return (
    <Present objUrl={presentARectObj} mtlUrl={presentARectMtl} {...props} />
  )
}

export function PresentARound(props) {
  return (
    <Present objUrl={presentARoundObj} mtlUrl={presentARoundMtl} {...props} />
  )
}

export function PresentBCube(props) {
  return (
    <Present objUrl={presentBCubeObj} mtlUrl={presentBCubeMtl} {...props} />
  )
}

export function PresentBRectangle(props) {
  return (
    <Present objUrl={presentBRectObj} mtlUrl={presentBRectMtl} {...props} />
  )
}

export function PresentBRound(props) {
  return (
    <Present objUrl={presentBRoundObj} mtlUrl={presentBRoundMtl} {...props} />
  )
}
