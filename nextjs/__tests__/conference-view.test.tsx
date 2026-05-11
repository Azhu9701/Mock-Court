import { describe, it, expect } from 'vitest'
import { render } from '@testing-library/react'
import { ConferenceView } from '@/components/conference-view'

describe('ConferenceView', () => {
  it('renders without crashing', () => {
    const { getByTestId } = render(
      <ConferenceView
        messages={{}}
        synthesis=""
        collisions={[]}
        toolCalls={[]}
      />
    )
    expect(getByTestId('conference-view')).toBeInTheDocument()
  })
})
