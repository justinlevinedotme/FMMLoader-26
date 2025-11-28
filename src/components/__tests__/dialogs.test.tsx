import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { ModMetadataDialog } from '@/components/ModMetadataDialog';
import { ConflictsDialog } from '@/components/ConflictsDialog';
import { GraphicsPackConfirmDialog } from '@/components/GraphicsPackConfirmDialog';
import { I18nProvider } from '@/lib/i18n';
import type { GraphicsPackAnalysis } from '@/types';

const { mockDetectModType, mockCheckConflicts } = vi.hoisted(() => {
  return {
    mockDetectModType: vi.fn().mockResolvedValue('graphics'),
    mockCheckConflicts: vi
      .fn()
      .mockResolvedValue([
        { file_path: 'graphics/picture.png', conflicting_mods: ['Mod A', 'Mod B'] },
      ]),
  };
});

vi.mock('@/hooks/useTauri', () => ({
  tauriCommands: {
    detectModType: mockDetectModType,
    checkConflicts: mockCheckConflicts,
  },
}));

describe('Dialog components', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('ModMetadataDialog submits when required fields are filled', async () => {
    const onSubmit = vi.fn();

    render(
      <I18nProvider locale="en">
        <ModMetadataDialog
          open
          onOpenChange={() => {}}
          sourcePath="/mods/sample.zip"
          onSubmit={onSubmit}
        />
      </I18nProvider>
    );

    // Auto-detect sets mod type; ensure inputs are fillable
    await waitFor(() => expect(mockDetectModType).toHaveBeenCalled());

    await userEvent.clear(screen.getByLabelText(/Name/i));
    await userEvent.type(screen.getByLabelText(/Name/i), 'My Cool Mod');
    await userEvent.clear(screen.getByLabelText(/Version/i));
    await userEvent.type(screen.getByLabelText(/Version/i), '2.0.0');

    await userEvent.click(screen.getByRole('button', { name: /Save/i }));

    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({
        name: 'My Cool Mod',
        version: '2.0.0',
        mod_type: 'graphics',
      })
    );
  });

  it('ConflictsDialog renders conflict entries when open', async () => {
    render(<ConflictsDialog open onOpenChange={() => {}} onDisableMod={vi.fn()} />);

    await waitFor(() => expect(mockCheckConflicts).toHaveBeenCalled());

    expect(await screen.findByText(/graphics\/picture\.png/i)).toBeInTheDocument();
    expect(screen.getByText('Mod A')).toBeInTheDocument();
    expect(screen.getByText('Mod B')).toBeInTheDocument();
  });

  it('GraphicsPackConfirmDialog shows suggested path and allows selection', async () => {
    const analysis: GraphicsPackAnalysis = {
      pack_type: 'Faces',
      confidence: 0.8,
      suggested_paths: ['faces/sample-pack'],
      file_count: 2,
      total_size_bytes: 2048,
      has_config_xml: true,
      subdirectory_breakdown: { faces: 2 },
      is_flat_pack: true,
    };

    render(
      <GraphicsPackConfirmDialog
        analysis={analysis}
        onConfirm={vi.fn()}
        onCancel={() => {}}
        userDirPath="/Users/alice/FM26"
      />
    );

    expect(screen.getByText(/graphics\/faces\/sample-pack/i)).toBeInTheDocument();

    expect(screen.getByText('/Users/alice/FM26/graphics/faces/sample-pack')).toBeInTheDocument();
  });
});
