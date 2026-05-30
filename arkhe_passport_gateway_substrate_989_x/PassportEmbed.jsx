// PassportEmbed.jsx — Substrato 989.x.2
// Componente React embarcável para verificação de identidade ARKHE
// Cross-links: 989.x, 958 (Clarity-Gate), 983 (API-Gateway)
// Deities: Hermes, Iris, Themis
// Arquiteto ORCID: 0009-0005-2697-4668

import React, { useState, useCallback, useEffect } from 'react';
import PropTypes from 'prop-types';

/**
 * ClarityGate — Substrato 958
 * Verifica se a landing page passa no teste de clareza:
 * 1. O que é? (What is it?)
 * 2. É pra mim? (Is it for me?)
 * 3. Por que agora? (Why now?)
 */
const CLARITY_QUESTIONS = [
  "O que é a ARKHE Code Cathedral?",
  "Esta verificação é para mim?",
  "Por que verificar minha identidade agora?"
];

const CLARITY_TERMS_BANNED = [
  "blockchain", "web3", "disruptive", "synergy", "paradigm",
  "leverage", "scalable", "decentralized"  // buzzwords proibidos pelo Clarity-Gate
];

const CLARITY_VERBS_APPROVED = [
  "verificar", "provar", "confirmar", "garantir", "proteger"
];

/**
 * PassportEmbed — Componente principal
 *
 * Props:
 *   apiBaseUrl: string    — URL base da API Gateway (983)
 *   scorerId: string      — ID do Passport Scorer
 *   onVerified: function  — Callback quando verificado
 *   onError: function     — Callback em erro
 *   theme: 'light' | 'dark' | 'cathedral'
 */
export default function PassportEmbed({
  apiBaseUrl = "https://api.arkhe-cathedral.org/v1",
  scorerId = "1",
  onVerified = null,
  onError = null,
  theme = 'cathedral'
}) {
  const [address, setAddress] = useState('');
  const [orcidId, setOrcidId] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);
  const [clarityScore, setClarityScore] = useState(0);

  // Clarity-Gate: verificar se o texto do usuário passa no teste
  const checkClarity = useCallback((text) => {
    const lower = text.toLowerCase();
    const hasBanned = CLARITY_TERMS_BANNED.some(term => lower.includes(term));
    const hasApproved = CLARITY_VERBS_APPROVED.some(verb => lower.includes(verb));
    const score = hasBanned ? 0 : (hasApproved ? 1 : 0.5);
    return { passed: score >= 0.5, score };
  }, []);

  const verifyIdentity = useCallback(async () => {
    if (!address || !address.startsWith('0x')) {
      setError('Endereço EVM inválido. Deve começar com 0x.');
      return;
    }

    // Clarity-Gate check
    const clarity = checkClarity(address);
    setClarityScore(clarity.score);
    if (!clarity.passed) {
      setError('Clarity-Gate (958): Endereço contém termos proibidos ou não passou no teste de clareza.');
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      // 1. Verificar humanidade via Passport Gateway (989.x)
      const humanityRes = await fetch(
        `${apiBaseUrl}/identity/passport?address=${address}${orcidId ? `&orcid_id=${orcidId}` : ''}`
      );

      if (!humanityRes.ok) {
        throw new Error(`Passport API error: ${humanityRes.status}`);
      }

      const humanityData = await humanityRes.json();

      // 2. Verificar Proof of Clean Hands (989.x.1)
      const cleanHandsRes = await fetch(
        `${apiBaseUrl}/clean-hands/check?address=${address}`
      );

      const cleanHandsData = cleanHandsRes.ok ? await cleanHandsRes.json() : { risk_level: 'unknown' };

      // 3. Verificar se pode votar na DAO (979)
      const daoRes = await fetch(
        `${apiBaseUrl}/dao/verify-voter?address=${address}`
      );

      const daoData = daoRes.ok ? await daoRes.json() : { can_vote: false };

      // 4. Compilar resultado
      const verificationResult = {
        address,
        orcidId: orcidId || null,
        isHuman: humanityData.is_human,
        humanityScore: humanityData.score,
        humanitySeal: humanityData.seal,
        stamps: humanityData.stamps?.length || 0,
        orcidVerified: humanityData.orcid_verified,
        riskLevel: cleanHandsData.risk_level,
        riskScore: cleanHandsData.score,
        cleanHandsSeal: cleanHandsData.seal,
        canVote: daoData.can_vote,
        clarityScore,
        timestamp: new Date().toISOString(),
        substrate: "989.x",
        seal: `VERIFY-${humanityData.seal?.replace('HP-', '') || 'UNKNOWN'}`,
      };

      setResult(verificationResult);

      if (onVerified && verificationResult.isHuman && verificationResult.canVote) {
        onVerified(verificationResult);
      }
    } catch (err) {
      setError(err.message);
      if (onError) onError(err);
    } finally {
      setLoading(false);
    }
  }, [address, orcidId, apiBaseUrl, onVerified, onError, checkClarity, clarityScore]);

  // Temas
  const themes = {
    light: {
      bg: '#ffffff',
      text: '#1a1a2e',
      accent: '#16213e',
      border: '#e0e0e0',
      button: '#0f3460',
      buttonText: '#ffffff',
      success: '#16c79a',
      warning: '#e7d40a',
      error: '#e94560',
    },
    dark: {
      bg: '#0f0f23',
      text: '#e0e0e0',
      accent: '#16213e',
      border: '#1a1a3e',
      button: '#e94560',
      buttonText: '#ffffff',
      success: '#16c79a',
      warning: '#e7d40a',
      error: '#ff6b6b',
    },
    cathedral: {
      bg: '#0a0a1a',
      text: '#d4af37',  // ouro
      accent: '#1a1a3e',
      border: '#2a2a5e',
      button: '#d4af37',
      buttonText: '#0a0a1a',
      success: '#7fff00',  // chartreuse
      warning: '#ffd700',
      error: '#ff4500',
    }
  };

  const t = themes[theme] || themes.cathedral;

  return (
    <div style={{
      fontFamily: "'Cinzel', 'Georgia', serif",
      backgroundColor: t.bg,
      color: t.text,
      border: `2px solid ${t.border}`,
      borderRadius: '8px',
      padding: '24px',
      maxWidth: '480px',
      margin: '0 auto',
      boxShadow: theme === 'cathedral' ? '0 0 20px rgba(212, 175, 55, 0.1)' : '0 4px 6px rgba(0,0,0,0.1)',
    }}>
      {/* Header */}
      <div style={{ textAlign: 'center', marginBottom: '20px' }}>
        <h2 style={{
          margin: 0,
          fontSize: '1.5rem',
          letterSpacing: '0.1em',
          textTransform: 'uppercase',
        }}>
          ⚜ Verificação de Identidade
        </h2>
        <p style={{
          margin: '8px 0 0',
          fontSize: '0.8rem',
          opacity: 0.7,
          fontFamily: "'Courier New', monospace",
        }}>
          Substrato 989.x — PASSPORT-GATEWAY
        </p>
      </div>

      {/* Clarity-Gate Badge */}
      <div style={{
        backgroundColor: t.accent,
        borderRadius: '4px',
        padding: '8px 12px',
        marginBottom: '16px',
        fontSize: '0.75rem',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
      }}>
        <span>🔒 Clarity-Gate (958)</span>
        <span style={{
          color: clarityScore >= 0.5 ? t.success : t.error,
          fontWeight: 'bold',
        }}>
          {clarityScore >= 0.5 ? 'PASS' : 'PENDING'}
        </span>
      </div>

      {/* Input: Address */}
      <div style={{ marginBottom: '16px' }}>
        <label style={{
          display: 'block',
          marginBottom: '6px',
          fontSize: '0.85rem',
          fontWeight: 'bold',
        }}>
          Endereço EVM (0x...)
        </label>
        <input
          type="text"
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          placeholder="0x..."
          style={{
            width: '100%',
            padding: '10px 12px',
            backgroundColor: t.accent,
            color: t.text,
            border: `1px solid ${t.border}`,
            borderRadius: '4px',
            fontFamily: "'Courier New', monospace",
            fontSize: '0.9rem',
            boxSizing: 'border-box',
          }}
        />
      </div>

      {/* Input: ORCID (optional) */}
      <div style={{ marginBottom: '20px' }}>
        <label style={{
          display: 'block',
          marginBottom: '6px',
          fontSize: '0.85rem',
          fontWeight: 'bold',
        }}>
          ORCID (opcional)
        </label>
        <input
          type="text"
          value={orcidId}
          onChange={(e) => setOrcidId(e.target.value)}
          placeholder="0000-0000-0000-0000"
          style={{
            width: '100%',
            padding: '10px 12px',
            backgroundColor: t.accent,
            color: t.text,
            border: `1px solid ${t.border}`,
            borderRadius: '4px',
            fontFamily: "'Courier New', monospace",
            fontSize: '0.9rem',
            boxSizing: 'border-box',
          }}
        />
        <p style={{ margin: '4px 0 0', fontSize: '0.7rem', opacity: 0.6 }}>
          Vinculação acadêmica via Substrato 982
        </p>
      </div>

      {/* Verify Button */}
      <button
        onClick={verifyIdentity}
        disabled={loading}
        style={{
          width: '100%',
          padding: '12px',
          backgroundColor: t.button,
          color: t.buttonText,
          border: 'none',
          borderRadius: '4px',
          fontFamily: "'Cinzel', 'Georgia', serif",
          fontSize: '1rem',
          fontWeight: 'bold',
          cursor: loading ? 'not-allowed' : 'pointer',
          opacity: loading ? 0.6 : 1,
          letterSpacing: '0.05em',
          textTransform: 'uppercase',
        }}
      >
        {loading ? '⏳ Verificando...' : '⚜ Verificar Identidade'}
      </button>

      {/* Error */}
      {error && (
        <div style={{
          marginTop: '16px',
          padding: '12px',
          backgroundColor: `${t.error}20`,
          border: `1px solid ${t.error}`,
          borderRadius: '4px',
          color: t.error,
          fontSize: '0.85rem',
        }}>
          ✗ {error}
        </div>
      )}

      {/* Result */}
      {result && (
        <div style={{
          marginTop: '16px',
          padding: '16px',
          backgroundColor: t.accent,
          border: `1px solid ${result.isHuman ? t.success : t.warning}`,
          borderRadius: '4px',
        }}>
          <h3 style={{
            margin: '0 0 12px',
            fontSize: '1.1rem',
            color: result.isHuman ? t.success : t.warning,
          }}>
            {result.isHuman ? '✓ Humano Verificado' : '⚠ Verificação Inconclusiva'}
          </h3>

          <div style={{ fontSize: '0.8rem', fontFamily: "'Courier New', monospace" }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
              <span>Score:</span>
              <span style={{ color: result.humanityScore >= 0.75 ? t.success : t.warning }}>
                {(result.humanityScore * 100).toFixed(1)}%
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
              <span>Stamps:</span>
              <span>{result.stamps}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
              <span>ORCID:</span>
              <span>{result.orcidVerified ? '✓' : '✗'}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
              <span>Risco AML:</span>
              <span style={{
                color: result.riskLevel === 'clear' ? t.success :
                       result.riskLevel === 'low' ? t.warning : t.error
              }}>
                {result.riskLevel.toUpperCase()}
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
              <span>DAO Vote:</span>
              <span>{result.canVote ? '✓ Autorizado' : '✗ Bloqueado'}</span>
            </div>
            <div style={{
              marginTop: '12px',
              paddingTop: '8px',
              borderTop: `1px solid ${t.border}`,
              fontSize: '0.7rem',
              opacity: 0.7,
              wordBreak: 'break-all',
            }}>
              Seal: {result.seal}
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <div style={{
        marginTop: '20px',
        textAlign: 'center',
        fontSize: '0.65rem',
        opacity: 0.5,
        fontFamily: "'Courier New', monospace",
      }}>
        <p>ARKHE Code Cathedral v∞.Ω.∇+++</p>
        <p>Substrato 989.x — Seal: 989-PASSPORT-GATEWAY-4B3CB68C02D21E5A</p>
        <p>Cross-links: 958, 982, 983, 954, 979, 923</p>
      </div>
    </div>
  );
}

PassportEmbed.propTypes = {
  apiBaseUrl: PropTypes.string,
  scorerId: PropTypes.string,
  onVerified: PropTypes.func,
  onError: PropTypes.func,
  theme: PropTypes.oneOf(['light', 'dark', 'cathedral']),
};
